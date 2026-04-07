use std::any::Any;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SignalId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum DependencyId {
    Signal(SignalId),
    Computed(ScopeId),
}

#[derive(Clone)]
pub struct Signal<T> {
    id: SignalId,
    marker: PhantomData<(T, Rc<()>)>,
}

#[derive(Clone)]
pub struct Computed<T> {
    scope: ScopeId,
    marker: PhantomData<(T, Rc<()>)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectHandle {
    scope: ScopeId,
    marker: PhantomData<Rc<()>>,
}

pub fn signal<T: Clone + 'static>(value: T) -> Signal<T> {
    RUNTIME.with(|runtime| runtime.borrow_mut().create_signal(value))
}

pub fn computed<T, F>(compute: F) -> Computed<T>
where
    T: Clone + 'static,
    F: Fn() -> T + 'static,
{
    RUNTIME.with(|runtime| runtime.borrow_mut().create_computed(compute))
}

pub fn effect<F>(callback: F) -> EffectHandle
where
    F: FnMut() + 'static,
{
    let scope = RUNTIME.with(|runtime| runtime.borrow_mut().create_effect(callback));
    run_effect(scope);
    EffectHandle {
        scope,
        marker: PhantomData,
    }
}

pub fn flush_reactivity() {
    while let Some(scope) = RUNTIME.with(|runtime| runtime.borrow_mut().next_dirty_effect()) {
        run_effect(scope);
    }
}

pub fn reset_reactivity_for_testing() {
    RUNTIME.with(|runtime| *runtime.borrow_mut() = ReactiveRuntime::default());
}

pub fn render_in_component_scope<T, F>(existing: Option<ScopeId>, render: F) -> (ScopeId, T)
where
    F: FnOnce(ScopeId) -> T,
{
    let scope = RUNTIME.with(|runtime| runtime.borrow_mut().prepare_component_scope(existing));
    let value = render(scope);
    RUNTIME.with(|runtime| runtime.borrow_mut().finish_component_scope(scope));
    (scope, value)
}

pub fn take_dirty_component_scopes() -> Vec<ScopeId> {
    RUNTIME.with(|runtime| runtime.borrow_mut().take_dirty_components())
}

pub fn dispose_component_scope(scope: ScopeId) {
    RUNTIME.with(|runtime| runtime.borrow_mut().dispose_component(scope));
}

pub(crate) fn dispose_computed_scope(scope: ScopeId) {
    RUNTIME.with(|runtime| runtime.borrow_mut().dispose_scope(scope));
}

pub fn current_reactive_scope() -> Option<ScopeId> {
    RUNTIME.with(|runtime| runtime.borrow().active_scopes.last().copied())
}

impl<T: Clone + 'static> Signal<T> {
    pub fn get(&self) -> T {
        RUNTIME.with(|runtime| runtime.borrow_mut().read_signal(self.id))
    }

    pub fn set(&self, value: T) {
        RUNTIME.with(|runtime| runtime.borrow_mut().write_signal(self.id, value));
    }

    pub fn set_if_changed(&self, value: T)
    where
        T: PartialEq,
    {
        RUNTIME.with(|runtime| {
            runtime.borrow_mut().write_signal_if_changed(self.id, value);
        });
    }

    pub fn update<F>(&self, update: F)
    where
        F: FnOnce(&mut T),
    {
        RUNTIME.with(|runtime| runtime.borrow_mut().update_signal(self.id, update));
    }
}

impl<T: Clone + 'static> Computed<T> {
    pub fn get(&self) -> T {
        RUNTIME.with(|runtime| {
            runtime
                .borrow_mut()
                .record_dependency(DependencyId::Computed(self.scope));
        });

        let evaluator =
            RUNTIME.with(|runtime| runtime.borrow_mut().prepare_computed_evaluation(self.scope));

        if let Some(evaluator) = evaluator {
            // Evaluate outside the runtime borrow so nested `get()` calls can
            // register dependencies without tripping RefCell reentrancy.
            let value = evaluator();
            RUNTIME.with(|runtime| {
                runtime
                    .borrow_mut()
                    .finish_computed_evaluation(self.scope, value);
            });
        }

        RUNTIME.with(|runtime| runtime.borrow().read_cached_computed(self.scope))
    }

    pub(crate) fn scope_id(&self) -> ScopeId {
        self.scope
    }
}

impl EffectHandle {
    pub fn stop(&self) {
        RUNTIME.with(|runtime| runtime.borrow_mut().dispose_effect(self.scope));
    }
}

thread_local! {
    static RUNTIME: RefCell<ReactiveRuntime> = RefCell::new(ReactiveRuntime::default());
}

fn run_effect(scope: ScopeId) {
    let effect = RUNTIME.with(|runtime| runtime.borrow_mut().prepare_effect_run(scope));
    if let Some(effect) = effect {
        // Like computeds, effects run outside the graph borrow so reads can
        // rebuild their dependency edges during execution.
        (effect.borrow_mut())();
        RUNTIME.with(|runtime| runtime.borrow_mut().finish_effect_run(scope));
    }
}

#[derive(Default)]
struct ReactiveRuntime {
    next_signal_id: usize,
    next_scope_id: usize,
    active_scopes: Vec<ScopeId>,
    signals: HashMap<SignalId, Box<dyn Any>>,
    signal_dependents: HashMap<SignalId, BTreeSet<ScopeId>>,
    computed_dependents: HashMap<ScopeId, BTreeSet<ScopeId>>,
    scope_dependencies: HashMap<ScopeId, BTreeSet<DependencyId>>,
    scopes: HashMap<ScopeId, ScopeState>,
    dirty_effects: VecDeque<ScopeId>,
    dirty_components: VecDeque<ScopeId>,
}

enum ScopeState {
    Computed(ComputedState),
    Effect(EffectState),
    Component(ComponentState),
}

struct ComputedState {
    evaluator: Rc<dyn Fn() -> Box<dyn Any>>,
    cached: Option<Box<dyn Any>>,
    dirty: bool,
    evaluating: bool,
}

struct EffectState {
    effect: Rc<RefCell<Box<dyn FnMut()>>>,
    dirty: bool,
    queued: bool,
    disposed: bool,
}

struct ComponentState {
    dirty: bool,
    queued: bool,
}

impl ReactiveRuntime {
    fn create_signal<T: Clone + 'static>(&mut self, value: T) -> Signal<T> {
        let id = SignalId(self.next_signal_id);
        self.next_signal_id += 1;
        self.signals.insert(id, Box::new(value));
        Signal {
            id,
            marker: PhantomData,
        }
    }

    fn create_computed<T, F>(&mut self, compute: F) -> Computed<T>
    where
        T: Clone + 'static,
        F: Fn() -> T + 'static,
    {
        let scope = ScopeId(self.next_scope_id);
        self.next_scope_id += 1;
        let evaluator: Rc<dyn Fn() -> Box<dyn Any>> = Rc::new(move || Box::new(compute()));
        self.scopes.insert(
            scope,
            ScopeState::Computed(ComputedState {
                evaluator,
                cached: None,
                dirty: true,
                evaluating: false,
            }),
        );
        Computed {
            scope,
            marker: PhantomData,
        }
    }

    fn create_effect<F>(&mut self, effect: F) -> ScopeId
    where
        F: FnMut() + 'static,
    {
        let scope = ScopeId(self.next_scope_id);
        self.next_scope_id += 1;
        self.scopes.insert(
            scope,
            ScopeState::Effect(EffectState {
                effect: Rc::new(RefCell::new(Box::new(effect))),
                dirty: true,
                queued: false,
                disposed: false,
            }),
        );
        scope
    }

    fn prepare_component_scope(&mut self, existing: Option<ScopeId>) -> ScopeId {
        let scope = existing.unwrap_or_else(|| {
            let scope = ScopeId(self.next_scope_id);
            self.next_scope_id += 1;
            self.scopes.insert(
                scope,
                ScopeState::Component(ComponentState {
                    dirty: false,
                    queued: false,
                }),
            );
            scope
        });

        self.clear_scope_dependencies(scope);
        if let Some(ScopeState::Component(state)) = self.scopes.get_mut(&scope) {
            state.dirty = false;
            state.queued = false;
        }
        self.active_scopes.push(scope);
        scope
    }

    fn finish_component_scope(&mut self, scope: ScopeId) {
        let popped = self.active_scopes.pop();
        assert_eq!(popped, Some(scope), "component scope stack out of sync");
    }

    fn read_signal<T: Clone + 'static>(&mut self, id: SignalId) -> T {
        self.record_dependency(DependencyId::Signal(id));
        self.signals
            .get(&id)
            .expect("signal should exist")
            .downcast_ref::<T>()
            .expect("signal type mismatch")
            .clone()
    }

    fn write_signal<T: Clone + 'static>(&mut self, id: SignalId, value: T) {
        let signal = self
            .signals
            .get_mut(&id)
            .expect("signal should exist")
            .downcast_mut::<T>()
            .expect("signal type mismatch");
        *signal = value;
        self.propagate_dirty(DependencyId::Signal(id));
    }

    fn write_signal_if_changed<T>(&mut self, id: SignalId, value: T)
    where
        T: Clone + PartialEq + 'static,
    {
        let signal = self
            .signals
            .get_mut(&id)
            .expect("signal should exist")
            .downcast_mut::<T>()
            .expect("signal type mismatch");

        if *signal == value {
            return;
        }

        *signal = value;
        self.propagate_dirty(DependencyId::Signal(id));
    }

    fn update_signal<T: Clone + 'static, F>(&mut self, id: SignalId, update: F)
    where
        F: FnOnce(&mut T),
    {
        let signal = self
            .signals
            .get_mut(&id)
            .expect("signal should exist")
            .downcast_mut::<T>()
            .expect("signal type mismatch");
        update(signal);
        self.propagate_dirty(DependencyId::Signal(id));
    }

    fn record_dependency(&mut self, dependency: DependencyId) {
        let Some(&scope) = self.active_scopes.last() else {
            return;
        };

        if matches!(dependency, DependencyId::Computed(source) if source == scope) {
            return;
        }

        self.scope_dependencies
            .entry(scope)
            .or_default()
            .insert(dependency);

        match dependency {
            DependencyId::Signal(signal) => {
                self.signal_dependents
                    .entry(signal)
                    .or_default()
                    .insert(scope);
            }
            DependencyId::Computed(computed) => {
                self.computed_dependents
                    .entry(computed)
                    .or_default()
                    .insert(scope);
            }
        }
    }

    fn prepare_computed_evaluation(
        &mut self,
        scope: ScopeId,
    ) -> Option<Rc<dyn Fn() -> Box<dyn Any>>> {
        let should_evaluate = match self.scopes.get(&scope) {
            Some(ScopeState::Computed(state)) => state.dirty || state.cached.is_none(),
            _ => panic!("scope should be a computed"),
        };

        if !should_evaluate {
            return None;
        }

        let evaluating = match self.scopes.get(&scope) {
            Some(ScopeState::Computed(state)) => state.evaluating,
            _ => unreachable!(),
        };
        assert!(!evaluating, "computed cycle detected");

        self.clear_scope_dependencies(scope);

        let evaluator = match self.scopes.get_mut(&scope) {
            Some(ScopeState::Computed(state)) => {
                state.evaluating = true;
                state.evaluator.clone()
            }
            _ => unreachable!(),
        };

        self.active_scopes.push(scope);
        Some(evaluator)
    }

    fn finish_computed_evaluation(&mut self, scope: ScopeId, value: Box<dyn Any>) {
        match self.scopes.get_mut(&scope) {
            Some(ScopeState::Computed(state)) => {
                state.cached = Some(value);
                state.dirty = false;
                state.evaluating = false;
            }
            _ => panic!("scope should be a computed"),
        }

        let popped = self.active_scopes.pop();
        assert_eq!(popped, Some(scope), "computed scope stack out of sync");
    }

    fn read_cached_computed<T: Clone + 'static>(&self, scope: ScopeId) -> T {
        match self.scopes.get(&scope) {
            Some(ScopeState::Computed(state)) => state
                .cached
                .as_ref()
                .expect("computed should have a cached value")
                .downcast_ref::<T>()
                .expect("computed type mismatch")
                .clone(),
            _ => panic!("scope should be a computed"),
        }
    }

    fn prepare_effect_run(&mut self, scope: ScopeId) -> Option<Rc<RefCell<Box<dyn FnMut()>>>> {
        let should_run = match self.scopes.get(&scope) {
            Some(ScopeState::Effect(state)) => state.dirty && !state.disposed,
            _ => false,
        };

        if !should_run {
            return None;
        }

        self.clear_scope_dependencies(scope);

        let effect = match self.scopes.get_mut(&scope) {
            Some(ScopeState::Effect(state)) => {
                state.dirty = false;
                state.queued = false;
                state.effect.clone()
            }
            _ => unreachable!(),
        };

        self.active_scopes.push(scope);
        Some(effect)
    }

    fn finish_effect_run(&mut self, scope: ScopeId) {
        let popped = self.active_scopes.pop();
        assert_eq!(popped, Some(scope), "effect scope stack out of sync");
    }

    fn next_dirty_effect(&mut self) -> Option<ScopeId> {
        while let Some(scope) = self.dirty_effects.pop_front() {
            let ready = match self.scopes.get_mut(&scope) {
                Some(ScopeState::Effect(state)) => {
                    state.queued = false;
                    state.dirty && !state.disposed
                }
                _ => false,
            };
            if ready {
                return Some(scope);
            }
        }
        None
    }

    fn dispose_effect(&mut self, scope: ScopeId) {
        self.clear_scope_dependencies(scope);
        if let Some(ScopeState::Effect(state)) = self.scopes.get_mut(&scope) {
            state.disposed = true;
            state.dirty = false;
            state.queued = false;
        }
    }

    fn dispose_component(&mut self, scope: ScopeId) {
        if matches!(self.scopes.get(&scope), Some(ScopeState::Component(_))) {
            self.dispose_scope(scope);
        }
    }

    fn dispose_scope(&mut self, scope: ScopeId) {
        self.clear_scope_dependencies(scope);
        self.computed_dependents.remove(&scope);
        self.scope_dependencies.remove(&scope);
        self.scopes.remove(&scope);
    }

    fn clear_scope_dependencies(&mut self, scope: ScopeId) {
        let Some(dependencies) = self.scope_dependencies.remove(&scope) else {
            return;
        };

        for dependency in dependencies {
            match dependency {
                DependencyId::Signal(signal) => {
                    if let Some(dependents) = self.signal_dependents.get_mut(&signal) {
                        dependents.remove(&scope);
                    }
                }
                DependencyId::Computed(computed) => {
                    if let Some(dependents) = self.computed_dependents.get_mut(&computed) {
                        dependents.remove(&scope);
                    }
                }
            }
        }
    }

    fn take_dirty_components(&mut self) -> Vec<ScopeId> {
        let mut dirty = Vec::new();
        while let Some(scope) = self.dirty_components.pop_front() {
            let ready = match self.scopes.get_mut(&scope) {
                Some(ScopeState::Component(state)) => {
                    state.queued = false;
                    if state.dirty {
                        state.dirty = false;
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if ready {
                dirty.push(scope);
            }
        }
        dirty
    }

    fn propagate_dirty(&mut self, dependency: DependencyId) {
        let mut queue = VecDeque::from([dependency]);
        let mut visited_scopes = BTreeSet::new();

        while let Some(current) = queue.pop_front() {
            // Dirty state propagates through the dependency graph: signals
            // invalidate computeds, and invalid computeds wake downstream
            // computeds or effects without eagerly recomputing values.
            let dependents = match current {
                DependencyId::Signal(signal) => self
                    .signal_dependents
                    .get(&signal)
                    .cloned()
                    .unwrap_or_default(),
                DependencyId::Computed(computed) => self
                    .computed_dependents
                    .get(&computed)
                    .cloned()
                    .unwrap_or_default(),
            };

            for scope in dependents {
                if !visited_scopes.insert(scope) {
                    continue;
                }

                match self.scopes.get_mut(&scope) {
                    Some(ScopeState::Computed(state)) => {
                        state.dirty = true;
                        queue.push_back(DependencyId::Computed(scope));
                    }
                    Some(ScopeState::Effect(state)) => {
                        if state.disposed {
                            continue;
                        }
                        state.dirty = true;
                        if !state.queued {
                            state.queued = true;
                            self.dirty_effects.push_back(scope);
                        }
                    }
                    Some(ScopeState::Component(state)) => {
                        state.dirty = true;
                        if !state.queued {
                            state.queued = true;
                            self.dirty_components.push_back(scope);
                        }
                    }
                    None => {}
                }
            }
        }
    }
}

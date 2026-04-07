use std::{
    any::{Any, TypeId},
    cell::RefCell,
    rc::Rc,
};

use crate::reactivity::dispose_computed_scope;
use crate::{Computed, EffectHandle, HistoryBlock, HistoryEntry, Signal, computed, signal};

pub enum RuntimeRequest<Message> {
    EmitMessage(Message),
    CommitHistory(HistoryEntry),
    SetFocusScope(Option<String>),
    Quit,
}

#[derive(Default)]
pub struct HookStore {
    // Component-local signals, computeds, and effects are all stored as hook
    // slots so they keep stable identities across rerenders.
    slots: Vec<Box<dyn HookSlot>>,
    cursor: usize,
}

impl HookStore {
    pub fn begin_render(&mut self) {
        self.cursor = 0;
    }

    pub fn signal<T, F>(&mut self, init: F) -> Signal<T>
    where
        T: Clone + 'static,
        F: FnOnce() -> T,
    {
        let slot = self.cursor;
        self.cursor += 1;

        if slot == self.slots.len() {
            self.slots.push(Box::new(StateSlot {
                signal: signal(init()),
            }));
        }

        assert!(
            matches!(
                self.slots[slot].kind(),
                HookSlotKind::State(type_id) if type_id == TypeId::of::<T>()
            ),
            "hook type mismatch"
        );

        *self.slots[slot]
            .clone_state_signal()
            .expect("state hooks should expose a signal handle")
            .downcast::<Signal<T>>()
            .expect("hook type mismatch")
    }

    pub fn computed<T, F>(&mut self, compute: F) -> Computed<T>
    where
        T: Clone + 'static,
        F: Fn() -> T + 'static,
    {
        let slot = self.cursor;
        self.cursor += 1;

        if slot == self.slots.len() {
            self.slots.push(Box::new(ComputedSlot {
                computed: computed(compute),
            }));
        }

        assert!(
            matches!(
                self.slots[slot].kind(),
                HookSlotKind::Computed(type_id) if type_id == TypeId::of::<T>()
            ),
            "hook type mismatch"
        );

        *self.slots[slot]
            .clone_boxed_value()
            .downcast::<Computed<T>>()
            .expect("hook type mismatch")
    }

    pub fn effect<F>(&mut self, effect: F)
    where
        F: FnMut() + 'static,
    {
        let slot = self.cursor;
        self.cursor += 1;

        if slot == self.slots.len() {
            self.slots.push(Box::new(EffectSlot::new(effect)));
            return;
        }

        assert!(
            matches!(self.slots[slot].kind(), HookSlotKind::Effect),
            "hook type mismatch"
        );
    }

    pub fn finish_render(&mut self) {
        // Hooks beyond the last slot used this render have effectively been
        // unmounted, so tear them down before dropping the slots.
        while self.slots.len() > self.cursor {
            let mut slot = self.slots.pop().expect("slot should exist");
            slot.teardown();
        }
    }
}

trait HookSlot {
    fn kind(&self) -> HookSlotKind;
    fn clone_boxed_value(&self) -> Box<dyn Any>;
    fn clone_state_signal(&self) -> Option<Box<dyn Any>> {
        None
    }
    fn teardown(&mut self) {}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HookSlotKind {
    State(TypeId),
    Computed(TypeId),
    Effect,
}

struct StateSlot<T> {
    signal: Signal<T>,
}

impl<T: Clone + 'static> HookSlot for StateSlot<T> {
    fn kind(&self) -> HookSlotKind {
        HookSlotKind::State(TypeId::of::<T>())
    }

    fn clone_boxed_value(&self) -> Box<dyn Any> {
        Box::new(self.signal.get())
    }

    fn clone_state_signal(&self) -> Option<Box<dyn Any>> {
        Some(Box::new(self.signal.clone()))
    }
}

struct ComputedSlot<T> {
    computed: Computed<T>,
}

impl<T: Clone + 'static> HookSlot for ComputedSlot<T> {
    fn kind(&self) -> HookSlotKind {
        HookSlotKind::Computed(TypeId::of::<T>())
    }

    fn clone_boxed_value(&self) -> Box<dyn Any> {
        Box::new(self.computed.clone())
    }

    fn teardown(&mut self) {
        dispose_computed_scope(self.computed.scope_id());
    }
}

struct EffectSlot {
    handle: EffectHandle,
}

impl EffectSlot {
    fn new<F>(callback_fn: F) -> Self
    where
        F: FnMut() + 'static,
    {
        let callback = Rc::new(RefCell::new(Box::new(callback_fn) as Box<dyn FnMut()>));
        Self {
            handle: crate::effect(move || (callback.borrow_mut())()),
        }
    }
}

impl HookSlot for EffectSlot {
    fn kind(&self) -> HookSlotKind {
        HookSlotKind::Effect
    }

    fn clone_boxed_value(&self) -> Box<dyn Any> {
        panic!("effect slots do not expose values")
    }

    fn teardown(&mut self) {
        self.handle.stop();
    }
}

impl Drop for HookStore {
    fn drop(&mut self) {
        for slot in &mut self.slots {
            slot.teardown();
        }
    }
}

pub struct ViewCtx<'a, Message> {
    hooks: &'a mut HookStore,
    marker: std::marker::PhantomData<Message>,
}

pub type Cx<'a, Message> = ViewCtx<'a, Message>;

impl<'a, Message: Send + 'static> ViewCtx<'a, Message> {
    pub fn new(hooks: &'a mut HookStore) -> Self {
        Self {
            hooks,
            marker: std::marker::PhantomData,
        }
    }

    pub fn signal<T, F>(&mut self, init: F) -> Signal<T>
    where
        T: Clone + 'static,
        F: FnOnce() -> T,
    {
        self.hooks.signal(init)
    }

    pub fn effect<F>(&mut self, effect: F)
    where
        F: FnMut() + 'static,
    {
        // Install the effect once per hook slot. This avoids duplicate
        // subscriptions while the component rerenders around the same stable
        // reactive sources.
        self.hooks.effect(effect);
    }

    pub fn computed<T, F>(&mut self, compute: F) -> Computed<T>
    where
        T: Clone + 'static,
        F: Fn() -> T + 'static,
    {
        self.hooks.computed(compute)
    }
}

impl<Message> RuntimeRequest<Message> {
    pub fn commit_text(content: String) -> Self {
        Self::CommitHistory(HistoryEntry::Text(content))
    }

    pub fn commit_block(block: HistoryBlock) -> Self {
        Self::CommitHistory(HistoryEntry::Block(block))
    }
}

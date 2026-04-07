use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    future::Future,
};

use ansiq_core::{
    ComponentRenderer, Element, ElementKind, HistoryBlock, HistoryEntry, HookStore, Node, Rect,
    RuntimeRequest, ScopeId, ViewCtx, dispose_component_scope, flush_reactivity,
    render_in_component_scope, take_dirty_component_scopes,
};
use ansiq_layout::{layout_tree_with_ids, measure_node_height, relayout_tree_along_paths};
use ansiq_surface::Key;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender, error::SendError};

use crate::{FocusState, routing};

pub trait App {
    type Message: Send + 'static;

    fn mount(&mut self, _handle: &RuntimeHandle<Self::Message>) {}

    fn render(&mut self, cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message>;

    fn update(&mut self, message: Self::Message, handle: &RuntimeHandle<Self::Message>);

    fn on_unhandled_key(&mut self, _key: Key, _handle: &RuntimeHandle<Self::Message>) -> bool {
        false
    }
}

pub struct RuntimeHandle<Message> {
    sender: UnboundedSender<RuntimeRequest<Message>>,
}

impl<Message> Clone for RuntimeHandle<Message> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<Message: Send + 'static> RuntimeHandle<Message> {
    pub fn emit(&self, message: Message) -> Result<(), SendError<RuntimeRequest<Message>>> {
        self.sender.send(RuntimeRequest::EmitMessage(message))
    }

    pub fn quit(&self) -> Result<(), SendError<RuntimeRequest<Message>>> {
        self.sender.send(RuntimeRequest::Quit)
    }

    pub fn trap_focus_in(
        &self,
        scope_key: impl Into<String>,
    ) -> Result<(), SendError<RuntimeRequest<Message>>> {
        self.sender
            .send(RuntimeRequest::SetFocusScope(Some(scope_key.into())))
    }

    pub fn clear_focus_scope(&self) -> Result<(), SendError<RuntimeRequest<Message>>> {
        self.sender.send(RuntimeRequest::SetFocusScope(None))
    }

    pub fn commit_history(
        &self,
        content: String,
    ) -> Result<(), SendError<RuntimeRequest<Message>>> {
        self.sender.send(RuntimeRequest::commit_text(content))
    }

    pub fn commit_history_block(
        &self,
        block: HistoryBlock,
    ) -> Result<(), SendError<RuntimeRequest<Message>>> {
        self.sender.send(RuntimeRequest::commit_block(block))
    }

    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(future)
    }
}

pub struct Engine<A: App> {
    app: A,
    hooks: HookStore,
    component_hooks: HashMap<ScopeId, HookStore>,
    sender: UnboundedSender<RuntimeRequest<A::Message>>,
    receiver: UnboundedReceiver<RuntimeRequest<A::Message>>,
    focus: FocusState,
    pending_history: Vec<HistoryEntry>,
    tree: Option<Node<A::Message>>,
    redraw_regions: Option<Vec<Rect>>,
    root_scope: Option<ScopeId>,
    component_scopes: BTreeSet<ScopeId>,
    pending_component_scopes: Vec<ScopeId>,
    bounds: Rect,
    required_height: u16,
    next_node_id: usize,
    needs_rerender: bool,
    dirty: bool,
    mounted: bool,
}

impl<A: App> Engine<A> {
    pub fn new(app: A) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        Self {
            app,
            hooks: HookStore::default(),
            component_hooks: HashMap::new(),
            sender,
            receiver,
            focus: FocusState::default(),
            pending_history: Vec::new(),
            tree: None,
            redraw_regions: None,
            root_scope: None,
            component_scopes: BTreeSet::new(),
            pending_component_scopes: Vec::new(),
            bounds: Rect::new(0, 0, 80, 24),
            required_height: 1,
            next_node_id: 0,
            needs_rerender: true,
            dirty: true,
            mounted: false,
        }
    }

    pub fn app(&self) -> &A {
        &self.app
    }

    pub fn app_mut(&mut self) -> &mut A {
        &mut self.app
    }

    pub fn handle(&self) -> RuntimeHandle<A::Message> {
        RuntimeHandle {
            sender: self.sender.clone(),
        }
    }

    pub fn mount(&mut self) {
        if self.mounted {
            return;
        }

        let handle = self.handle();
        self.app.mount(&handle);
        self.mounted = true;
        self.needs_rerender = true;
        self.dirty = true;
    }

    pub fn tree(&self) -> Option<&Node<A::Message>> {
        self.tree.as_ref()
    }

    pub fn redraw_regions(&self) -> Option<&[Rect]> {
        self.redraw_regions.as_deref()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty || self.needs_rerender
    }

    pub fn focused(&self) -> Option<usize> {
        self.focus.current()
    }

    pub fn take_pending_history(&mut self) -> Vec<HistoryEntry> {
        std::mem::take(&mut self.pending_history)
    }

    pub fn required_height(&self) -> u16 {
        self.required_height
    }

    pub fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
        self.needs_rerender = true;
        self.dirty = true;
    }

    pub fn render_tree(&mut self) {
        self.sync_reactivity();

        if !self.needs_rerender && self.pending_component_scopes.is_empty() && self.tree.is_some() {
            if self.dirty {
                self.redraw_regions = None;
                self.dirty = false;
            }
            return;
        }

        let root_scope_dirty = self
            .root_scope
            .is_some_and(|scope| self.pending_component_scopes.contains(&scope));

        if self.tree.is_none()
            || self.root_scope.is_none()
            || self.needs_rerender
            || root_scope_dirty
        {
            self.render_root_tree();
            return;
        }

        let dirty_scopes: BTreeSet<_> = std::mem::take(&mut self.pending_component_scopes)
            .into_iter()
            .collect();
        if dirty_scopes.is_empty() {
            return;
        }

        if let Some(mut tree) = self.tree.take() {
            let focused_before = self.focus.current();
            let focus_rect_before =
                focused_before.and_then(|focused| find_node_rect(&tree, focused));
            let dirty_paths =
                self.rerender_dirty_component_nodes(&mut tree, &dirty_scopes, &mut Vec::new());
            let mut relayout = relayout_tree_along_paths(&mut tree, self.bounds, &dirty_paths);
            self.required_height = tree.measured_height;
            self.focus.sync_from_tree(&tree);
            let focused_after = self.focus.current();
            let focus_rect_after = focused_after.and_then(|focused| find_node_rect(&tree, focused));
            append_focus_transition_regions(
                &mut relayout.invalidated_regions,
                focus_rect_before,
                focus_rect_after,
            );
            self.redraw_regions = Some(relayout.invalidated_regions);
            self.refresh_component_scope_registry(&tree);
            self.tree = Some(tree);
        }

        self.dirty = false;
        self.needs_rerender = false;
    }

    pub fn handle_input(&mut self, key: Key) -> bool {
        let Some(tree) = self.tree.as_mut() else {
            return false;
        };

        let focused_before = self.focus.current();
        let effect = routing::handle_key(tree, &mut self.focus, key);
        if effect.dirty {
            self.dirty = true;
        }
        if self.focus.current() != focused_before {
            self.dirty = true;
        }
        if let Some(message) = effect.message {
            let handle = self.handle();
            self.app.update(message, &handle);
            self.needs_rerender = true;
            self.dirty = true;
        }
        if !effect.handled && !effect.quit {
            let handle = self.handle();
            if self.app.on_unhandled_key(key, &handle) {
                self.needs_rerender = true;
                self.dirty = true;
            }
        }

        effect.quit
    }

    pub fn drain_requests(&mut self) -> bool {
        let mut should_quit = false;

        while let Ok(request) = self.receiver.try_recv() {
            match request {
                RuntimeRequest::EmitMessage(message) => {
                    let handle = self.handle();
                    self.app.update(message, &handle);
                    self.needs_rerender = true;
                    self.dirty = true;
                }
                RuntimeRequest::CommitHistory(entry) => {
                    self.pending_history.push(entry);
                    self.dirty = true;
                }
                RuntimeRequest::SetFocusScope(scope_key) => {
                    self.focus.set_scope_key(scope_key);
                    if let Some(tree) = self.tree.as_ref() {
                        self.focus.sync_from_tree(tree);
                    }
                    self.dirty = true;
                }
                RuntimeRequest::Quit => should_quit = true,
            }
        }

        should_quit
    }

    fn sync_reactivity(&mut self) {
        flush_reactivity();
        let dirty = take_dirty_component_scopes();
        if !dirty.is_empty() {
            self.pending_component_scopes.extend(dirty);
        }
    }

    fn render_root_tree(&mut self) {
        let continuity_restore = self
            .tree
            .as_ref()
            .map(capture_widget_runtime_state_in_subtree)
            .unwrap_or_default();
        let focus_restore = self.focus.current().and_then(|current| {
            self.tree
                .as_ref()
                .map(|tree| capture_focus_continuity_in_subtree(tree, current))
        });

        self.hooks.begin_render();
        let (scope, element) = render_in_component_scope(self.root_scope, |_| {
            let element = {
                let mut cx = ViewCtx::new(&mut self.hooks);
                self.app.render(&mut cx)
            };
            self.hooks.finish_render();
            element
        });
        self.root_scope = Some(scope);

        let element = self.resolve_component_elements(element);
        self.next_node_id = 0;
        let mut tree = layout_tree_with_ids(element, self.bounds, &mut self.next_node_id);
        self.required_height = tree.measured_height;
        self.redraw_regions = None;
        restore_widget_runtime_state_in_subtree(&mut tree, &continuity_restore);
        if let Some(focus_target) = focus_restore
            && let Some(focus_id) = restore_focus_continuity_in_subtree(&tree, &focus_target)
        {
            self.focus.set_current(Some(focus_id));
        }
        self.focus.sync_from_tree(&tree);
        self.refresh_component_scope_registry(&tree);
        self.tree = Some(tree);
        self.pending_component_scopes.clear();
        self.dirty = false;
        self.needs_rerender = false;
    }

    fn resolve_component_elements(&mut self, element: Element<A::Message>) -> Element<A::Message> {
        let Element {
            kind,
            layout,
            style,
            focusable,
            continuity_key,
            children,
        } = element;

        match kind {
            ElementKind::Component(mut props) => {
                let sender = self.sender.clone();
                let renderer = props.renderer.clone();
                let (scope, child) = render_in_component_scope(props.scope, |scope| {
                    self.render_component(renderer, scope, sender)
                });
                props.scope = Some(scope);
                let child = self.resolve_component_elements(child);

                let mut element = Element::new(ElementKind::Component(props))
                    .with_layout(layout)
                    .with_style(style)
                    .with_focusable(focusable)
                    .with_children(vec![child]);
                if let Some(key) = continuity_key {
                    element = element.with_continuity_key(key);
                }
                element
            }
            kind => {
                let children = children
                    .into_iter()
                    .map(|child| self.resolve_component_elements(child))
                    .collect();
                let mut element = Element::new(kind)
                    .with_layout(layout)
                    .with_style(style)
                    .with_focusable(focusable)
                    .with_children(children);
                if let Some(key) = continuity_key {
                    element = element.with_continuity_key(key);
                }
                element
            }
        }
    }

    fn rerender_dirty_component_nodes(
        &mut self,
        node: &mut Node<A::Message>,
        dirty_scopes: &BTreeSet<ScopeId>,
        path: &mut Vec<usize>,
    ) -> Vec<Vec<usize>> {
        if let ElementKind::Component(props) = &mut node.element.kind
            && let Some(scope) = props.scope
            && dirty_scopes.contains(&scope)
        {
            let focus_restore = self.focus.current().and_then(|current| {
                node.children
                    .first()
                    .map(|child| capture_focus_continuity_in_subtree(child, current))
            });
            let widget_state_restore = node
                .children
                .first()
                .map(capture_widget_runtime_state_in_subtree)
                .unwrap_or_default();
            let sender = self.sender.clone();
            let renderer = props.renderer.clone();
            let (_, child) = render_in_component_scope(Some(scope), |scope| {
                self.render_component(renderer, scope, sender)
            });
            let child = self.resolve_component_elements(child);
            let mut child = layout_tree_with_ids(child, node.rect, &mut self.next_node_id);
            restore_widget_runtime_state_in_subtree(&mut child, &widget_state_restore);
            if let Some(focus_target) = focus_restore
                && let Some(focus_id) = restore_focus_continuity_in_subtree(&child, &focus_target)
            {
                self.focus.set_current(Some(focus_id));
            }
            node.children = vec![child];
            node.measured_height = 0;
            node.measured_height = measure_node_height(node, node.rect.width);
            return vec![path.clone()];
        }

        let mut dirty_paths = Vec::new();
        for (index, child) in node.children.iter_mut().enumerate() {
            path.push(index);
            dirty_paths.extend(self.rerender_dirty_component_nodes(child, dirty_scopes, path));
            path.pop();
        }
        dirty_paths
    }

    fn render_component(
        &mut self,
        renderer: ComponentRenderer<A::Message>,
        scope: ScopeId,
        _sender: UnboundedSender<RuntimeRequest<A::Message>>,
    ) -> Element<A::Message> {
        match renderer {
            ComponentRenderer::Static(renderer) => renderer(),
            ComponentRenderer::WithCx(renderer) => {
                let hooks = self.component_hooks.entry(scope).or_default();
                hooks.begin_render();
                let element = {
                    let mut cx = ViewCtx::new(hooks);
                    renderer(&mut cx)
                };
                hooks.finish_render();
                element
            }
        }
    }

    fn refresh_component_scope_registry(&mut self, tree: &Node<A::Message>) {
        let mut live = BTreeSet::new();
        collect_component_scopes(tree, &mut live);

        // Component scopes are long-lived reactive identities. When a subtree
        // disappears, drop its hook store and unregister the scope so stale
        // signals/watchers cannot keep waking removed UI.
        let stale: Vec<_> = self.component_scopes.difference(&live).copied().collect();
        for scope in stale {
            self.component_hooks.remove(&scope);
            self.pending_component_scopes
                .retain(|pending| *pending != scope);
            dispose_component_scope(scope);
        }

        self.component_scopes = live;
    }
}

fn collect_component_scopes<Message>(node: &Node<Message>, scopes: &mut BTreeSet<ScopeId>) {
    if let ElementKind::Component(props) = &node.element.kind
        && let Some(scope) = props.scope
    {
        scopes.insert(scope);
    }

    for child in &node.children {
        collect_component_scopes(child, scopes);
    }
}

fn find_node_rect<Message>(node: &Node<Message>, target_id: usize) -> Option<Rect> {
    if node.id == target_id {
        return Some(node.rect);
    }

    for child in &node.children {
        if let Some(rect) = find_node_rect(child, target_id) {
            return Some(rect);
        }
    }

    None
}

fn append_focus_transition_regions(
    regions: &mut Vec<Rect>,
    before: Option<Rect>,
    after: Option<Rect>,
) {
    if before == after {
        return;
    }

    if let Some(rect) = before {
        push_region_if_missing(regions, rect);
    }
    if let Some(rect) = after {
        push_region_if_missing(regions, rect);
    }
}

fn push_region_if_missing(regions: &mut Vec<Rect>, rect: Rect) {
    if regions.iter().any(|existing| *existing == rect) {
        return;
    }
    regions.push(rect);
}

#[derive(Debug, Clone, Default)]
struct FocusContinuityTarget {
    continuity_key: Option<String>,
    fallback_index: usize,
}

#[derive(Debug, Clone, Default)]
struct WidgetRuntimeStateSnapshot {
    keyed: BTreeMap<String, ansiq_core::RuntimeWidgetState>,
    ordered: Vec<ansiq_core::RuntimeWidgetState>,
}

fn focusable_index_in_subtree<Message>(node: &Node<Message>, target_id: usize) -> Option<usize> {
    let mut index = 0usize;
    focusable_index_in_subtree_inner(node, target_id, &mut index)
}

fn focusable_index_in_subtree_inner<Message>(
    node: &Node<Message>,
    target_id: usize,
    index: &mut usize,
) -> Option<usize> {
    if node.element.focusable {
        if node.id == target_id {
            return Some(*index);
        }
        *index = index.saturating_add(1);
    }

    for child in &node.children {
        if let Some(found) = focusable_index_in_subtree_inner(child, target_id, index) {
            return Some(found);
        }
    }

    None
}

fn focusable_id_at_index<Message>(node: &Node<Message>, target_index: usize) -> Option<usize> {
    let mut index = 0usize;
    focusable_id_at_index_inner(node, target_index, &mut index)
}

fn focusable_id_at_index_inner<Message>(
    node: &Node<Message>,
    target_index: usize,
    index: &mut usize,
) -> Option<usize> {
    if node.element.focusable {
        if *index == target_index {
            return Some(node.id);
        }
        *index = index.saturating_add(1);
    }

    for child in &node.children {
        if let Some(found) = focusable_id_at_index_inner(child, target_index, index) {
            return Some(found);
        }
    }

    None
}

fn capture_focus_continuity_in_subtree<Message>(
    node: &Node<Message>,
    target_id: usize,
) -> FocusContinuityTarget {
    FocusContinuityTarget {
        continuity_key: continuity_key_for_node_id(node, target_id),
        fallback_index: focusable_index_in_subtree(node, target_id).unwrap_or(0),
    }
}

fn continuity_key_for_node_id<Message>(node: &Node<Message>, target_id: usize) -> Option<String> {
    if node.id == target_id {
        return node.element.continuity_key.clone();
    }

    for child in &node.children {
        if let Some(key) = continuity_key_for_node_id(child, target_id) {
            return Some(key);
        }
    }

    None
}

fn focusable_id_for_continuity_key<Message>(
    node: &Node<Message>,
    target_key: &str,
) -> Option<usize> {
    if node.element.focusable && node.element.continuity_key() == Some(target_key) {
        return Some(node.id);
    }

    for child in &node.children {
        if let Some(id) = focusable_id_for_continuity_key(child, target_key) {
            return Some(id);
        }
    }

    None
}

fn restore_focus_continuity_in_subtree<Message>(
    node: &Node<Message>,
    target: &FocusContinuityTarget,
) -> Option<usize> {
    if let Some(key) = target.continuity_key.as_deref()
        && let Some(id) = focusable_id_for_continuity_key(node, key)
    {
        return Some(id);
    }

    focusable_id_at_index(node, target.fallback_index)
}

fn capture_widget_runtime_state_in_subtree<Message>(
    node: &Node<Message>,
) -> WidgetRuntimeStateSnapshot {
    let mut snapshot = WidgetRuntimeStateSnapshot::default();
    capture_widget_runtime_state_in_subtree_inner(node, &mut snapshot);
    snapshot
}

fn capture_widget_runtime_state_in_subtree_inner<Message>(
    node: &Node<Message>,
    output: &mut WidgetRuntimeStateSnapshot,
) {
    if let Some(state) = node.element.kind.capture_runtime_state() {
        if let Some(key) = node.element.continuity_key() {
            output.keyed.insert(key.to_string(), state);
        } else {
            output.ordered.push(state);
        }
    }

    for child in &node.children {
        capture_widget_runtime_state_in_subtree_inner(child, output);
    }
}

fn restore_widget_runtime_state_in_subtree<Message>(
    node: &mut Node<Message>,
    snapshot: &WidgetRuntimeStateSnapshot,
) {
    let mut index = 0usize;
    restore_widget_runtime_state_in_subtree_inner(node, snapshot, &mut index);
}

fn restore_widget_runtime_state_in_subtree_inner<Message>(
    node: &mut Node<Message>,
    snapshot: &WidgetRuntimeStateSnapshot,
    index: &mut usize,
) {
    let expects_runtime_state = node.element.kind.capture_runtime_state().is_some();
    if expects_runtime_state {
        let keyed_state = node
            .element
            .continuity_key()
            .and_then(|key| snapshot.keyed.get(key));
        let positional_state = snapshot.ordered.get(*index);
        if let Some(state) = keyed_state.or(positional_state) {
            node.element.kind.restore_runtime_state(state);
        } else {
            node.element.kind.initialize_runtime_state();
        }
        if keyed_state.is_none() {
            *index = index.saturating_add(1);
        }
    }

    for child in &mut node.children {
        restore_widget_runtime_state_in_subtree_inner(child, snapshot, index);
    }
}

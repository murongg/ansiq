use ansiq_core::Node;

#[derive(Debug, Default)]
pub struct FocusState {
    order: Vec<usize>,
    current: Option<usize>,
    scope_key: Option<String>,
}

impl FocusState {
    pub fn sync_from_tree<Message>(&mut self, tree: &Node<Message>) {
        let mut next_order = Vec::new();
        if let Some(scope_key) = self.scope_key.as_deref() {
            if let Some(scope_root) = find_scope_root(tree, scope_key) {
                collect_focusable_ids(scope_root, &mut next_order);
            } else {
                collect_focusable_ids(tree, &mut next_order);
            }
        } else {
            collect_focusable_ids(tree, &mut next_order);
        }

        self.current = if next_order.is_empty() {
            None
        } else if self
            .current
            .is_some_and(|current| next_order.contains(&current))
        {
            self.current
        } else {
            next_order.first().copied()
        };
        self.order = next_order;
    }

    pub fn current(&self) -> Option<usize> {
        self.current
    }

    pub fn set_current(&mut self, current: Option<usize>) {
        self.current = current;
    }

    pub fn set_scope_key(&mut self, scope_key: Option<String>) {
        self.scope_key = scope_key;
    }

    pub fn scope_key(&self) -> Option<&str> {
        self.scope_key.as_deref()
    }

    pub fn next(&mut self) {
        self.step(1);
    }

    pub fn prev(&mut self) {
        self.step_back();
    }

    fn step(&mut self, amount: usize) {
        if self.order.is_empty() {
            self.current = None;
            return;
        }

        let index = self
            .current
            .and_then(|current| self.order.iter().position(|id| *id == current))
            .unwrap_or(0);
        let next = (index + amount) % self.order.len();
        self.current = Some(self.order[next]);
    }

    fn step_back(&mut self) {
        if self.order.is_empty() {
            self.current = None;
            return;
        }

        let index = self
            .current
            .and_then(|current| self.order.iter().position(|id| *id == current))
            .unwrap_or(0);
        let next = if index == 0 {
            self.order.len() - 1
        } else {
            index - 1
        };
        self.current = Some(self.order[next]);
    }
}

fn collect_focusable_ids<Message>(node: &Node<Message>, output: &mut Vec<usize>) {
    if node.element.focusable {
        output.push(node.id);
    }

    for child in &node.children {
        collect_focusable_ids(child, output);
    }
}

fn find_scope_root<'a, Message>(
    node: &'a Node<Message>,
    scope_key: &str,
) -> Option<&'a Node<Message>> {
    if node.element.continuity_key() == Some(scope_key) {
        return Some(node);
    }

    for child in &node.children {
        if let Some(found) = find_scope_root(child, scope_key) {
            return Some(found);
        }
    }

    None
}

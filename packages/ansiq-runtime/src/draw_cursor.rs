use ansiq_core::{ElementKind, Node, display_width_prefix};

pub(crate) fn find_cursor_position<Message>(
    node: &Node<Message>,
    focused: usize,
) -> Option<(u16, u16)> {
    if node.id == focused {
        return input_cursor_position(node);
    }

    for child in &node.children {
        if let Some(position) = find_cursor_position(child, focused) {
            return Some(position);
        }
    }

    None
}

fn input_cursor_position<Message>(node: &Node<Message>) -> Option<(u16, u16)> {
    let ElementKind::Input(props) = &node.element.kind else {
        return None;
    };

    let inner = node.rect.shrink(1);
    if inner.width == 0 || inner.height == 0 {
        return None;
    }

    let content_width = display_width_prefix(&props.value, props.cursor);
    let cursor_x = inner
        .x
        .saturating_add(content_width)
        .min(inner.right().saturating_sub(1));
    Some((cursor_x, inner.y))
}

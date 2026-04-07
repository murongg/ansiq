use ansiq_core::{ElementKind, Node, WidgetKey, WidgetRouteContext};
use ansiq_surface::Key;
use unicode_width::UnicodeWidthChar;

use crate::FocusState;

#[derive(Debug, PartialEq, Eq)]
pub struct RouteEffect<Message> {
    pub handled: bool,
    pub dirty: bool,
    pub message: Option<Message>,
    pub quit: bool,
}

impl<Message> Default for RouteEffect<Message> {
    fn default() -> Self {
        Self {
            handled: false,
            dirty: false,
            message: None,
            quit: false,
        }
    }
}

pub fn handle_key<Message>(
    tree: &mut Node<Message>,
    focus: &mut FocusState,
    key: Key,
) -> RouteEffect<Message> {
    if matches!(key, Key::CtrlC) {
        return RouteEffect {
            handled: true,
            quit: true,
            ..RouteEffect::default()
        };
    }

    if let Some(id) = focus.current() {
        if let Some(node) = find_node_mut(tree, id) {
            let effect = route_to_node(node, key);
            if effect.handled || effect.dirty || effect.message.is_some() || effect.quit {
                return effect;
            }
        }
    }

    match key {
        Key::Tab | Key::Down | Key::Char('j') => {
            focus.next();
            return RouteEffect {
                handled: true,
                ..RouteEffect::default()
            };
        }
        Key::BackTab | Key::Up | Key::Char('k') => {
            focus.prev();
            return RouteEffect {
                handled: true,
                ..RouteEffect::default()
            };
        }
        _ => {}
    }

    RouteEffect::default()
}

fn route_to_node<Message>(node: &mut Node<Message>, key: Key) -> RouteEffect<Message> {
    let Some(widget_key) = map_widget_key(key) else {
        return RouteEffect::default();
    };

    node.element
        .kind
        .route_widget_key(
            widget_key,
            WidgetRouteContext {
                viewport_height: node.rect.height as usize,
                scroll_view_max_offset: scroll_view_max_offset(node),
            },
        )
        .map(|effect| RouteEffect {
            handled: true,
            dirty: effect.dirty,
            message: effect.message,
            quit: false,
        })
        .unwrap_or_default()
}

fn find_node_mut<Message>(node: &mut Node<Message>, id: usize) -> Option<&mut Node<Message>> {
    if node.id == id {
        return Some(node);
    }

    for child in &mut node.children {
        if let Some(found) = find_node_mut(child, id) {
            return Some(found);
        }
    }

    None
}

fn scroll_view_max_offset<Message>(node: &Node<Message>) -> Option<usize> {
    let child = node.children.first()?;
    let (content, _) = scroll_text_content(child)?;
    let lines = wrap_lines(&content, node.rect.width);
    Some(lines.len().saturating_sub(node.rect.height as usize))
}

fn map_widget_key(key: Key) -> Option<WidgetKey> {
    match key {
        Key::Up => Some(WidgetKey::Up),
        Key::Down => Some(WidgetKey::Down),
        Key::Left => Some(WidgetKey::Left),
        Key::Right => Some(WidgetKey::Right),
        Key::Esc => Some(WidgetKey::Escape),
        Key::Enter => Some(WidgetKey::Enter),
        Key::Backspace => Some(WidgetKey::Backspace),
        Key::Char(ch) => Some(WidgetKey::Char(ch)),
        _ => None,
    }
}

fn scroll_text_content<Message>(node: &Node<Message>) -> Option<(String, ())> {
    match &node.element.kind {
        ElementKind::StreamingText(props) => Some((props.content.clone(), ())),
        ElementKind::Text(props) => Some((props.content.clone(), ())),
        ElementKind::Paragraph(props) => Some((props.content.plain(), ())),
        _ => None,
    }
}

fn wrap_lines(content: &str, width: u16) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }

    let mut lines = Vec::new();

    for raw_line in content.split('\n') {
        if raw_line.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current = String::new();
        let mut current_width = 0u16;

        for ch in raw_line.chars() {
            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
            if current_width.saturating_add(char_width) > width && !current.is_empty() {
                lines.push(current);
                current = String::new();
                current_width = 0;
            }

            current.push(ch);
            current_width = current_width.saturating_add(char_width.max(1));
        }

        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

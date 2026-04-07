use std::sync::{Arc, Mutex};

use ansiq_core::{
    BoxProps, Constraint, Direction, Element, ElementKind, HighlightSpacing, InputProps, Layout,
    Length, Line, ListDirection, ListProps, ListState, Node, Rect, Row, ScrollViewProps,
    ScrollbarOrientation, ScrollbarProps, ScrollbarState, Span, TableAlignment, TableProps,
    TableState, TabsProps,
};
use ansiq_layout::layout_tree;
use ansiq_runtime::{FocusState, handle_key};
use ansiq_surface::Key;

fn input<Message>(
    value: &str,
    on_change: Option<ansiq_core::ChangeHandler>,
    on_submit: Option<ansiq_core::SubmitHandler<Message>>,
) -> Element<Message> {
    Element::new(ElementKind::Input(InputProps {
        value: value.to_string(),
        placeholder: String::new(),
        on_change,
        on_submit,
        cursor: value.chars().count(),
    }))
    .with_layout(Layout {
        width: Length::Fill,
        height: Length::Fixed(3),
    })
    .with_focusable(true)
}

fn input_value<Message>(node: &ansiq_core::Node<Message>) -> &str {
    match &node.element.kind {
        ElementKind::Input(props) => &props.value,
        other => panic!("expected input, got {other:?}"),
    }
}

fn input_cursor<Message>(node: &ansiq_core::Node<Message>) -> usize {
    match &node.element.kind {
        ElementKind::Input(props) => props.cursor,
        other => panic!("expected input, got {other:?}"),
    }
}

fn list<Message>(
    items: &[&str],
    selected: Option<usize>,
    on_select: Option<ansiq_core::SelectHandler<Message>>,
) -> Element<Message> {
    Element::new(ElementKind::List(ListProps {
        block: None,
        items: items.iter().map(|item| (*item).into()).collect(),
        state: ListState::default().with_selected(selected),
        highlight_symbol: Some(Line::from(">>")),
        highlight_style: Default::default(),
        highlight_spacing: HighlightSpacing::WhenSelected,
        repeat_highlight_symbol: false,
        direction: ListDirection::TopToBottom,
        scroll_padding: 0,
        on_select,
    }))
    .with_layout(Layout {
        width: Length::Fill,
        height: Length::Auto,
    })
    .with_focusable(true)
}

fn tabs<Message>(
    titles: &[&str],
    selected: Option<usize>,
    on_select: Option<ansiq_core::SelectHandler<Message>>,
) -> Element<Message> {
    Element::new(ElementKind::Tabs(TabsProps {
        block: None,
        titles: titles.iter().map(|title| (*title).into()).collect(),
        selected,
        selection_explicit: false,
        highlight_style: Default::default(),
        divider: Span::raw("|"),
        padding_left: Line::from(" "),
        padding_right: Line::from(" "),
        on_select,
    }))
    .with_layout(Layout {
        width: Length::Fill,
        height: Length::Fixed(1),
    })
    .with_focusable(true)
}

fn table<Message>(
    rows: &[Vec<&str>],
    selected: Option<usize>,
    on_select: Option<ansiq_core::SelectHandler<Message>>,
) -> Element<Message> {
    Element::new(ElementKind::Table(TableProps {
        block: None,
        header: Some(Row::new(["Name", "Status"])),
        footer: None,
        rows: rows
            .iter()
            .map(|row| Row::new(row.iter().copied()))
            .collect(),
        widths: vec![Constraint::Fill(1), Constraint::Fill(1)],
        column_spacing: 1,
        flex: ansiq_core::Flex::Start,
        alignments: vec![TableAlignment::Left, TableAlignment::Left],
        state: TableState::default().with_selected(selected),
        highlight_symbol: Some(">>".into()),
        row_highlight_style: Default::default(),
        column_highlight_style: Default::default(),
        cell_highlight_style: Default::default(),
        highlight_spacing: HighlightSpacing::WhenSelected,
        on_select,
    }))
    .with_layout(Layout {
        width: Length::Fill,
        height: Length::Auto,
    })
    .with_focusable(true)
}

fn scroll_view<Message>(
    content: &str,
    follow_bottom: bool,
    offset: Option<usize>,
    on_scroll: Option<ansiq_core::ScrollHandler<Message>>,
) -> Element<Message> {
    Element::new(ElementKind::ScrollView(ScrollViewProps {
        follow_bottom,
        offset,
        on_scroll,
    }))
    .with_layout(Layout {
        width: Length::Fill,
        height: Length::Fixed(2),
    })
    .with_focusable(true)
    .with_children(vec![Element::new(ElementKind::StreamingText(
        ansiq_core::StreamingTextProps {
            content: content.to_string(),
        },
    ))])
}

fn scrollbar<Message>(
    position: usize,
    content_length: usize,
    viewport_length: usize,
    orientation: ScrollbarOrientation,
    on_scroll: Option<ansiq_core::ScrollHandler<Message>>,
) -> Element<Message> {
    Element::new(ElementKind::Scrollbar(ScrollbarProps {
        state: ScrollbarState::new(content_length)
            .position(position)
            .viewport_content_length(viewport_length),
        orientation,
        thumb_symbol: "█".to_string(),
        thumb_style: ansiq_core::Style::default(),
        track_symbol: Some("░".to_string()),
        track_style: ansiq_core::Style::default(),
        begin_symbol: Some("↑".to_string()),
        begin_style: ansiq_core::Style::default(),
        end_symbol: Some("↓".to_string()),
        end_style: ansiq_core::Style::default(),
        on_scroll,
    }))
    .with_layout(Layout {
        width: match orientation {
            ScrollbarOrientation::VerticalLeft | ScrollbarOrientation::VerticalRight => {
                Length::Fixed(1)
            }
            ScrollbarOrientation::HorizontalTop | ScrollbarOrientation::HorizontalBottom => {
                Length::Fill
            }
        },
        height: match orientation {
            ScrollbarOrientation::VerticalLeft | ScrollbarOrientation::VerticalRight => {
                Length::Fill
            }
            ScrollbarOrientation::HorizontalTop | ScrollbarOrientation::HorizontalBottom => {
                Length::Fixed(1)
            }
        },
    })
    .with_focusable(true)
}

#[test]
fn handle_key_updates_focused_input_and_emits_submit_message() {
    let changes = Arc::new(Mutex::new(Vec::new()));
    let mut tree: Node<usize> = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![input(
            "",
            Some(std::boxed::Box::new({
                let changes = Arc::clone(&changes);
                move |value| changes.lock().unwrap().push(value)
            })),
            Some(std::boxed::Box::new(|value| Some(value.len()))),
        )]),
        Rect::new(0, 0, 20, 3),
    );

    let mut focus = FocusState::default();
    focus.sync_from_tree(&tree);

    let effect = handle_key(&mut tree, &mut focus, Key::Char('h'));
    assert!(effect.dirty);
    assert_eq!(effect.message, None);
    assert_eq!(input_value(&tree.children[0]), "h");

    let effect = handle_key(&mut tree, &mut focus, Key::Char('i'));
    assert!(effect.dirty);
    assert_eq!(input_value(&tree.children[0]), "hi");
    assert_eq!(
        &*changes.lock().unwrap(),
        &["h".to_string(), "hi".to_string()]
    );

    let effect = handle_key(&mut tree, &mut focus, Key::Enter);
    assert_eq!(effect.message, Some(2));
    assert!(!effect.quit);
}

#[test]
fn tab_moves_focus_but_character_input_keeps_priority() {
    let first_changes = Arc::new(Mutex::new(Vec::new()));
    let second_changes = Arc::new(Mutex::new(Vec::new()));

    let mut tree: Node<()> = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![
            input(
                "",
                Some(std::boxed::Box::new({
                    let first_changes = Arc::clone(&first_changes);
                    move |value| first_changes.lock().unwrap().push(value)
                })),
                None,
            ),
            input(
                "",
                Some(std::boxed::Box::new({
                    let second_changes = Arc::clone(&second_changes);
                    move |value| second_changes.lock().unwrap().push(value)
                })),
                None,
            ),
        ]),
        Rect::new(0, 0, 20, 6),
    );

    let mut focus = FocusState::default();
    focus.sync_from_tree(&tree);

    let first_id = tree.children[0].id;
    let second_id = tree.children[1].id;
    assert_eq!(focus.current(), Some(first_id));

    let effect = handle_key(&mut tree, &mut focus, Key::Tab);
    assert!(!effect.dirty);
    assert_eq!(focus.current(), Some(second_id));

    let effect = handle_key(&mut tree, &mut focus, Key::Char('j'));
    assert!(effect.dirty);
    assert_eq!(focus.current(), Some(second_id));
    assert_eq!(&*first_changes.lock().unwrap(), &[] as &[String]);
    assert_eq!(&*second_changes.lock().unwrap(), &["j".to_string()]);
}

#[test]
fn ctrl_c_requests_runtime_quit() {
    let mut tree: Node<()> = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![input("", None, None)]),
        Rect::new(0, 0, 20, 3),
    );

    let mut focus = FocusState::default();
    focus.sync_from_tree(&tree);

    let effect = handle_key(&mut tree, &mut focus, Key::CtrlC);
    assert!(effect.quit);
    assert!(effect.handled);
}

#[test]
fn escape_is_left_unhandled_when_no_widget_consumes_it() {
    let mut tree: Node<()> = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![input("", None, None), input("", None, None)]),
        Rect::new(0, 0, 20, 6),
    );

    let mut focus = FocusState::default();
    focus.sync_from_tree(&tree);
    let first_id = tree.children[0].id;

    let effect = handle_key(&mut tree, &mut focus, Key::Esc);
    assert!(!effect.handled);
    assert!(!effect.dirty);
    assert_eq!(focus.current(), Some(first_id));
}

#[test]
fn left_and_right_move_the_input_cursor() {
    let mut tree: Node<()> = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![input("hello", None, None)]),
        Rect::new(0, 0, 20, 3),
    );

    let mut focus = FocusState::default();
    focus.sync_from_tree(&tree);

    assert_eq!(input_cursor(&tree.children[0]), 5);
    assert!(handle_key(&mut tree, &mut focus, Key::Left).dirty);
    assert_eq!(input_cursor(&tree.children[0]), 4);

    assert!(handle_key(&mut tree, &mut focus, Key::Left).dirty);
    assert_eq!(input_cursor(&tree.children[0]), 3);

    assert!(handle_key(&mut tree, &mut focus, Key::Right).dirty);
    assert_eq!(input_cursor(&tree.children[0]), 4);
}

#[test]
fn typing_and_backspace_edit_at_the_cursor_position() {
    let mut tree: Node<()> = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![input("hello", None, None)]),
        Rect::new(0, 0, 20, 3),
    );

    let mut focus = FocusState::default();
    focus.sync_from_tree(&tree);

    handle_key(&mut tree, &mut focus, Key::Left);
    handle_key(&mut tree, &mut focus, Key::Left);
    let effect = handle_key(&mut tree, &mut focus, Key::Char('X'));
    assert!(effect.dirty);
    assert_eq!(input_value(&tree.children[0]), "helXlo");
    assert_eq!(input_cursor(&tree.children[0]), 4);

    let effect = handle_key(&mut tree, &mut focus, Key::Backspace);
    assert!(effect.dirty);
    assert_eq!(input_value(&tree.children[0]), "hello");
    assert_eq!(input_cursor(&tree.children[0]), 3);
}

#[test]
fn list_consumes_arrow_keys_updates_selection_and_emits_message() {
    let mut tree: Node<usize> = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![list(
            &["src", "Cargo.toml", "README.md"],
            Some(0),
            Some(std::boxed::Box::new(Some)),
        )]),
        Rect::new(0, 0, 24, 5),
    );

    let mut focus = FocusState::default();
    focus.sync_from_tree(&tree);

    let effect = handle_key(&mut tree, &mut focus, Key::Down);
    assert!(effect.dirty);
    assert_eq!(effect.message, Some(1));
    match &tree.children[0].element.kind {
        ElementKind::List(props) => assert_eq!(props.state.selected(), Some(1)),
        other => panic!("expected list, got {other:?}"),
    }

    let effect = handle_key(&mut tree, &mut focus, Key::Enter);
    assert_eq!(effect.message, Some(1));
}

#[test]
fn tabs_use_horizontal_keys_to_change_the_selected_tab() {
    let mut tree: Node<usize> = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![tabs(
            &["Overview", "Files", "Logs"],
            Some(0),
            Some(std::boxed::Box::new(Some)),
        )]),
        Rect::new(0, 0, 24, 3),
    );

    let mut focus = FocusState::default();
    focus.sync_from_tree(&tree);

    let effect = handle_key(&mut tree, &mut focus, Key::Right);
    assert!(effect.dirty);
    assert_eq!(effect.message, Some(1));
    match &tree.children[0].element.kind {
        ElementKind::Tabs(props) => assert_eq!(props.selected, Some(1)),
        other => panic!("expected tabs, got {other:?}"),
    }

    let effect = handle_key(&mut tree, &mut focus, Key::Left);
    assert!(effect.dirty);
    assert_eq!(effect.message, Some(0));
}

#[test]
fn table_updates_selection_with_arrow_keys_and_keeps_focus_local() {
    let mut tree: Node<usize> = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![
            table(
                &[
                    vec!["ansiq", "ready"],
                    vec!["codexdemo", "streaming"],
                    vec!["agent", "idle"],
                ],
                Some(0),
                Some(std::boxed::Box::new(Some)),
            ),
            input("", None, None),
        ]),
        Rect::new(0, 0, 24, 8),
    );

    let mut focus = FocusState::default();
    focus.sync_from_tree(&tree);
    let table_id = tree.children[0].id;
    let input_id = tree.children[1].id;
    assert_eq!(focus.current(), Some(table_id));

    let effect = handle_key(&mut tree, &mut focus, Key::Down);
    assert!(effect.dirty);
    assert_eq!(effect.message, Some(1));
    assert_eq!(focus.current(), Some(table_id));

    let effect = handle_key(&mut tree, &mut focus, Key::Tab);
    assert!(!effect.dirty);
    assert_eq!(focus.current(), Some(input_id));
}

#[test]
fn table_scrolls_selection_window_when_the_selected_row_moves_past_visible_height() {
    let mut tree: Node<usize> = layout_tree(
        Element::new(ElementKind::Table(TableProps {
            block: None,
            header: Some(Row::new(["Name", "Status"])),
            footer: None,
            rows: vec![
                Row::new(["ansiq", "ready"]),
                Row::new(["codexdemo", "streaming"]),
                Row::new(["agent", "idle"]),
                Row::new(["worker", "busy"]),
            ],
            widths: vec![Constraint::Fill(1), Constraint::Fill(1)],
            column_spacing: 1,
            flex: ansiq_core::Flex::Start,
            alignments: vec![TableAlignment::Left, TableAlignment::Left],
            state: TableState::default().with_selected(Some(0)),
            highlight_symbol: Some(">>".into()),
            row_highlight_style: Default::default(),
            column_highlight_style: Default::default(),
            cell_highlight_style: Default::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            on_select: Some(std::boxed::Box::new(Some)),
        }))
        .with_layout(Layout {
            width: Length::Fill,
            height: Length::Fixed(3),
        })
        .with_focusable(true),
        Rect::new(0, 0, 24, 3),
    );

    let mut focus = FocusState::default();
    focus.sync_from_tree(&tree);

    assert_eq!(
        handle_key(&mut tree, &mut focus, Key::Down).message,
        Some(1)
    );
    assert_eq!(
        handle_key(&mut tree, &mut focus, Key::Down).message,
        Some(2)
    );
    assert_eq!(
        handle_key(&mut tree, &mut focus, Key::Down).message,
        Some(3)
    );

    match &tree.element.kind {
        ElementKind::Table(props) => {
            assert_eq!(props.state.selected(), Some(3));
            assert_eq!(props.state.offset(), 2);
        }
        other => panic!("expected table, got {other:?}"),
    }
}

#[test]
fn scroll_view_consumes_vertical_keys_and_emits_the_new_offset() {
    let mut tree: Node<usize> = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![
            scroll_view(
                "one\ntwo\nthree\nfour",
                false,
                Some(0),
                Some(std::boxed::Box::new(Some)),
            ),
            input("", None, None),
        ]),
        Rect::new(0, 0, 20, 5),
    );

    let mut focus = FocusState::default();
    focus.sync_from_tree(&tree);
    let scroll_id = tree.children[0].id;
    let input_id = tree.children[1].id;
    assert_eq!(focus.current(), Some(scroll_id));

    let effect = handle_key(&mut tree, &mut focus, Key::Down);
    assert!(effect.dirty);
    assert_eq!(effect.message, Some(1));
    assert_eq!(focus.current(), Some(scroll_id));
    match &tree.children[0].element.kind {
        ElementKind::ScrollView(props) => assert_eq!(props.offset, Some(1)),
        other => panic!("expected scroll view, got {other:?}"),
    }

    let effect = handle_key(&mut tree, &mut focus, Key::Tab);
    assert!(!effect.dirty);
    assert_eq!(focus.current(), Some(input_id));
}

#[test]
fn scrollbar_consumes_vertical_keys_and_emits_the_new_position() {
    let mut tree: Node<usize> = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![
            scrollbar(
                0,
                4,
                2,
                ScrollbarOrientation::VerticalRight,
                Some(std::boxed::Box::new(Some)),
            ),
            input("", None, None),
        ]),
        Rect::new(0, 0, 10, 5),
    );

    let mut focus = FocusState::default();
    focus.sync_from_tree(&tree);
    let scrollbar_id = tree.children[0].id;
    assert_eq!(focus.current(), Some(scrollbar_id));

    let effect = handle_key(&mut tree, &mut focus, Key::Down);
    assert!(effect.dirty);
    assert_eq!(effect.message, Some(1));
    match &tree.children[0].element.kind {
        ElementKind::Scrollbar(props) => {
            assert_eq!(props.state.get_position(), 1);
            assert_eq!(props.orientation, ScrollbarOrientation::VerticalRight);
        }
        other => panic!("expected scrollbar, got {other:?}"),
    }
}

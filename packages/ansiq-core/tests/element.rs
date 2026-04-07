use std::sync::{Arc, Mutex};

use ansiq_core::{
    Alignment, BlockProps, Borders, BoxProps, ChildLayoutKind, Color, Element, ElementKind,
    InputProps, IntoElement, Layout, Length, ListProps, ListState, Padding, PaneProps, Rect,
    ScrollDirection, ScrollViewProps, ScrollbarState, ShellProps, TextProps, TitlePosition,
    WidgetKey, WidgetRouteContext,
};

#[test]
fn new_element_uses_default_layout_and_no_children() {
    let element: Element<()> = Element::new(ElementKind::Text(TextProps {
        content: "ansiq".into(),
    }));

    assert_eq!(
        element.layout,
        Layout {
            width: Length::Fill,
            height: Length::Auto,
        }
    );
    assert!(element.children.is_empty());
    assert!(!element.focusable);
}

#[test]
fn text_helper_builds_text_element() {
    let element = Element::<()>::new_text("hello");

    assert_eq!(element.kind_name(), "Text");
}

#[test]
fn into_element_trait_passes_existing_elements_through() {
    let original = Element::<()>::new_text("hello");
    let converted = original.into_element();

    assert_eq!(converted.kind_name(), "Text");
}

#[test]
fn list_state_matches_ratatui_selection_semantics() {
    let mut state = ListState::default().with_offset(3).with_selected(Some(4));
    assert_eq!(state.offset(), 3);
    assert_eq!(state.selected(), Some(4));

    state.select(None);
    assert_eq!(state.selected(), None);
    assert_eq!(state.offset(), 0);

    state.select_previous();
    assert_eq!(state.selected(), Some(usize::MAX));

    state.select_next();
    assert_eq!(state.selected(), Some(usize::MAX));

    state.select_first();
    assert_eq!(state.selected(), Some(0));

    state.select_last();
    assert_eq!(state.selected(), Some(usize::MAX));
}

#[test]
fn scrollbar_state_matches_ratatui_navigation_semantics() {
    let mut state = ScrollbarState::default()
        .with_content_length(10)
        .with_position(4)
        .with_viewport_content_length(3);

    assert_eq!(state.content_length_value(), 10);
    assert_eq!(state.get_position(), 4);
    assert_eq!(state.viewport_content_length_value(), 3);

    state.prev();
    assert_eq!(state.get_position(), 3);

    state.next();
    assert_eq!(state.get_position(), 4);

    state.scroll(ScrollDirection::Forward);
    assert_eq!(state.get_position(), 5);

    state.scroll(ScrollDirection::Backward);
    assert_eq!(state.get_position(), 4);

    state.first();
    assert_eq!(state.get_position(), 0);

    state.last();
    assert_eq!(state.get_position(), 9);
}

#[test]
fn block_child_layout_spec_uses_inner_rect_and_column_stack() {
    let block: Element<()> = Element::new(ElementKind::Block(BlockProps {
        titles: vec![ansiq_core::BlockTitle::new("Header")],
        title_alignment: Alignment::Left,
        title_position: TitlePosition::Top,
        borders: Borders::ALL,
        border_type: ansiq_core::BorderType::Plain,
        border_set: None,
        padding: Padding::all(1),
        border_style: Default::default(),
        title_style: Default::default(),
    }))
    .with_children(vec![Element::new_text("body")]);

    let spec = block.child_layout_spec(Rect::new(0, 0, 20, 10));

    assert_eq!(spec.bounds, Rect::new(2, 2, 16, 6));
    assert_eq!(
        spec.kind,
        ChildLayoutKind::Stack {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }
    );
}

#[test]
fn pane_child_layout_spec_uses_inner_rect_and_fill_semantics() {
    let pane: Element<()> = Element::new(ElementKind::Pane(PaneProps {
        title: Some("Output".into()),
    }))
    .with_children(vec![Element::new_text("stream")]);

    let spec = pane.child_layout_spec(Rect::new(0, 0, 12, 6));

    assert_eq!(spec.bounds, Rect::new(1, 1, 10, 4));
    assert_eq!(spec.kind, ChildLayoutKind::Fill);
}

#[test]
fn shell_child_layout_spec_uses_shell_strategy() {
    let shell: Element<()> = Element::new(ElementKind::Shell(ShellProps)).with_children(vec![
        Element::new_text("header"),
        Element::new_text("body"),
        Element::new_text("footer"),
    ]);

    let spec = shell.child_layout_spec(Rect::new(0, 0, 20, 10));

    assert_eq!(spec.bounds, Rect::new(0, 0, 20, 10));
    assert_eq!(spec.kind, ChildLayoutKind::Shell);
}

#[test]
fn scroll_view_and_component_child_layout_specs_fill_bounds() {
    let scroll: Element<()> = Element::new(ElementKind::ScrollView(ScrollViewProps {
        follow_bottom: false,
        offset: Some(3),
        on_scroll: None,
    }))
    .with_children(vec![Element::new_text("body")]);
    let scroll_spec = scroll.child_layout_spec(Rect::new(2, 3, 14, 7));
    assert_eq!(scroll_spec.bounds, Rect::new(2, 3, 14, 7));
    assert_eq!(scroll_spec.kind, ChildLayoutKind::Fill);

    let component = ansiq_core::component("Panel", || {
        Element::<()>::new(ElementKind::Box(BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(vec![Element::new_text("body")])
    });
    let component_spec = component.child_layout_spec(Rect::new(1, 2, 9, 4));
    assert_eq!(component_spec.bounds, Rect::new(1, 2, 9, 4));
    assert_eq!(component_spec.kind, ChildLayoutKind::Fill);
}

#[test]
fn shell_intrinsic_height_sums_child_heights() {
    let shell: Element<()> = Element::new(ElementKind::Shell(ShellProps))
        .with_children(vec![Element::new_text("header"), Element::new_text("body")]);

    assert_eq!(shell.intrinsic_height(20, &[2, 5]), 7);
}

#[test]
fn block_intrinsic_height_adds_border_title_and_padding_to_children() {
    let block: Element<()> = Element::new(ElementKind::Block(BlockProps {
        titles: vec![ansiq_core::BlockTitle::new("Header")],
        title_alignment: Alignment::Left,
        title_position: TitlePosition::Top,
        borders: Borders::ALL,
        border_type: ansiq_core::BorderType::Plain,
        border_set: None,
        padding: Padding::all(1),
        border_style: Default::default(),
        title_style: Default::default(),
    }))
    .with_children(vec![Element::new_text("body")]);

    assert_eq!(block.intrinsic_height(20, &[3]), 7);
}

#[test]
fn scroll_view_intrinsic_height_tracks_only_the_first_child() {
    let scroll: Element<()> = Element::new(ElementKind::ScrollView(ScrollViewProps {
        follow_bottom: false,
        offset: Some(3),
        on_scroll: None,
    }))
    .with_children(vec![
        Element::new_text("body"),
        Element::new_text("ignored"),
    ]);

    assert_eq!(scroll.intrinsic_height(20, &[4, 9]), 4);
}

#[test]
fn layout_only_containers_only_invalidate_themselves_when_styled() {
    let plain_box: Element<()> = Element::new(ElementKind::Box(BoxProps {
        direction: ansiq_core::Direction::Column,
        gap: 0,
    }));
    assert!(!plain_box.invalidates_self_on_layout_change());

    let styled_box: Element<()> = Element::new(ElementKind::Box(BoxProps {
        direction: ansiq_core::Direction::Column,
        gap: 0,
    }))
    .with_style(Color::Blue.into());
    assert!(styled_box.invalidates_self_on_layout_change());

    let component = ansiq_core::component("Panel", || Element::<()>::new_text("body"));
    assert!(!component.invalidates_self_on_layout_change());
}

#[test]
fn list_route_widget_key_updates_selection_and_emits_message() {
    let mut kind = ElementKind::List(ListProps {
        block: None,
        items: vec!["alpha".into(), "beta".into(), "gamma".into()],
        state: ListState::default().with_selected(Some(0)),
        highlight_symbol: None,
        highlight_style: Default::default(),
        highlight_spacing: Default::default(),
        repeat_highlight_symbol: false,
        direction: Default::default(),
        scroll_padding: 0,
        on_select: Some(Box::new(|index| Some(index))),
    });

    let effect = kind
        .route_widget_key(
            WidgetKey::Down,
            WidgetRouteContext {
                viewport_height: 2,
                scroll_view_max_offset: None,
            },
        )
        .expect("list should handle down");

    assert!(effect.dirty);
    assert_eq!(effect.message, Some(1));
    match kind {
        ElementKind::List(props) => {
            assert_eq!(props.state.selected(), Some(1));
            assert_eq!(props.state.offset(), 0);
        }
        _ => panic!("expected list"),
    }
}

#[test]
fn input_route_widget_key_edits_value_and_cursor() {
    let changes = Arc::new(Mutex::new(Vec::new()));
    let mut kind = ElementKind::Input(InputProps {
        value: String::new(),
        placeholder: String::new(),
        on_change: Some(Box::new({
            let changes = Arc::clone(&changes);
            move |value| changes.lock().unwrap().push(value)
        })),
        on_submit: Some(Box::new(|value| Some(value))),
        cursor: 0,
    });

    let insert = kind
        .route_widget_key(WidgetKey::Char('h'), WidgetRouteContext::default())
        .expect("input should handle char");
    assert!(insert.dirty);

    let submit = kind
        .route_widget_key(WidgetKey::Enter, WidgetRouteContext::default())
        .expect("input should handle enter");
    assert_eq!(submit.message, Some("h".to_string()));

    match kind {
        ElementKind::Input(props) => {
            assert_eq!(props.value, "h");
            assert_eq!(props.cursor, 1);
        }
        _ => panic!("expected input"),
    }
    assert_eq!(&*changes.lock().unwrap(), &["h".to_string()]);
}

#[test]
fn scroll_view_route_widget_key_updates_offset_within_bounds() {
    let mut kind = ElementKind::ScrollView(ScrollViewProps {
        follow_bottom: false,
        offset: Some(1),
        on_scroll: Some(Box::new(|offset| Some(offset))),
    });

    let effect = kind
        .route_widget_key(
            WidgetKey::Down,
            WidgetRouteContext {
                viewport_height: 0,
                scroll_view_max_offset: Some(3),
            },
        )
        .expect("scroll view should handle down");

    assert!(effect.dirty);
    assert_eq!(effect.message, Some(2));
    match kind {
        ElementKind::ScrollView(props) => assert_eq!(props.offset, Some(2)),
        _ => panic!("expected scroll view"),
    }
}

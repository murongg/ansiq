use ansiq_core::{
    BoxProps, Direction, Element, ElementKind, InputProps, Layout, Length, Rect, TextProps,
};
use ansiq_layout::layout_tree;
use ansiq_runtime::FocusState;

fn text(content: &str) -> Element<()> {
    Element::new(ElementKind::Text(TextProps {
        content: content.to_string(),
    }))
}

fn input(value: &str) -> Element<()> {
    Element::new(ElementKind::Input(InputProps {
        value: value.to_string(),
        placeholder: String::new(),
        on_change: None,
        on_submit: None,
        cursor: value.chars().count(),
    }))
    .with_layout(Layout {
        width: Length::Fill,
        height: Length::Fixed(3),
    })
    .with_focusable(true)
}

#[test]
fn focus_state_collects_focusable_nodes_in_preorder() {
    let tree = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![input("a"), text("label"), input("b")]),
        Rect::new(0, 0, 20, 7),
    );

    let mut focus = FocusState::default();
    focus.sync_from_tree(&tree);

    assert_eq!(focus.current(), Some(tree.children[0].id));
    focus.next();
    assert_eq!(focus.current(), Some(tree.children[2].id));
    focus.next();
    assert_eq!(focus.current(), Some(tree.children[0].id));
    focus.prev();
    assert_eq!(focus.current(), Some(tree.children[2].id));
}

#[test]
fn focus_state_preserves_current_node_when_tree_is_rebuilt() {
    let first_tree = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![input("a"), input("b")]),
        Rect::new(0, 0, 20, 6),
    );

    let mut focus = FocusState::default();
    focus.sync_from_tree(&first_tree);
    focus.next();

    let second_tree = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![input("a"), input("b")]),
        Rect::new(0, 0, 20, 6),
    );

    focus.sync_from_tree(&second_tree);
    assert_eq!(focus.current(), Some(second_tree.children[1].id));
}

#[test]
fn focus_state_can_trap_tab_order_inside_a_continuity_scoped_subtree() {
    let tree = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![
            Element::new(ElementKind::Box(BoxProps {
                direction: Direction::Column,
                gap: 0,
            }))
            .with_continuity_key("outside")
            .with_children(vec![input("a")]),
            Element::new(ElementKind::Box(BoxProps {
                direction: Direction::Column,
                gap: 0,
            }))
            .with_continuity_key("modal")
            .with_children(vec![input("b"), input("c")]),
        ]),
        Rect::new(0, 0, 20, 9),
    );

    let modal_first = tree.children[1].children[0].id;
    let modal_second = tree.children[1].children[1].id;

    let mut focus = FocusState::default();
    focus.set_scope_key(Some("modal".to_string()));
    focus.sync_from_tree(&tree);

    assert_eq!(focus.scope_key(), Some("modal"));
    assert_eq!(focus.current(), Some(modal_first));
    focus.next();
    assert_eq!(focus.current(), Some(modal_second));
    focus.next();
    assert_eq!(focus.current(), Some(modal_first));
}

use ansiq_core::{ElementKind, Node};
use ansiq_examples::scenarios::scroll_sync::ScrollSyncApp;
use ansiq_render::FrameBuffer;
use ansiq_runtime::{Engine, draw_tree};
use ansiq_surface::Key;

fn collect_text(node: &Node<()>, lines: &mut Vec<String>) {
    match &node.element.kind {
        ElementKind::Text(props) => lines.push(props.content.clone()),
        ElementKind::StatusBar(props) => lines.push(props.content.clone()),
        _ => {}
    }

    for child in &node.children {
        collect_text(child, lines);
    }
}

fn find_scroll_view(node: &Node<()>) -> Option<&Node<()>> {
    if matches!(node.element.kind, ElementKind::ScrollView(_)) {
        return Some(node);
    }

    node.children.iter().find_map(find_scroll_view)
}

fn find_scrollbar(node: &Node<()>) -> Option<&Node<()>> {
    if matches!(node.element.kind, ElementKind::Scrollbar(_)) {
        return Some(node);
    }

    node.children.iter().find_map(find_scrollbar)
}

fn rendered_screen(engine: &Engine<ScrollSyncApp>, width: u16, height: u16) -> String {
    let tree = engine.tree().expect("tree should exist");
    let mut buffer = FrameBuffer::new(width, height);
    draw_tree(tree, engine.focused(), &mut buffer);

    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer.get(x, y).symbol)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn scroll_sync_keeps_scroll_view_and_scrollbar_in_sync() {
    let mut engine = Engine::new(ScrollSyncApp::default());
    engine.render_tree();

    assert!(!engine.handle_input(Key::Down));
    engine.render_tree();

    let tree = engine.tree().expect("tree should exist");
    match &find_scroll_view(tree)
        .expect("scroll view should exist")
        .element
        .kind
    {
        ElementKind::ScrollView(props) => assert_eq!(props.offset, Some(1)),
        other => panic!("expected scroll view, got {other:?}"),
    }
    match &find_scrollbar(tree)
        .expect("scrollbar should exist")
        .element
        .kind
    {
        ElementKind::Scrollbar(props) => assert_eq!(props.state.get_position(), 1),
        other => panic!("expected scrollbar, got {other:?}"),
    }

    let mut lines = Vec::new();
    collect_text(tree, &mut lines);
    assert!(lines.join("\n").contains("offset: 1"));

    let screen = rendered_screen(&engine, 40, 8);
    assert!(screen.contains("two"));
    assert!(screen.contains("three"));
}

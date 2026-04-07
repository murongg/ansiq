use ansiq_core::{ElementKind, Node};
use ansiq_examples::scenarios::table_interaction::TableInteractionApp;
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

fn rendered_screen(engine: &Engine<TableInteractionApp>, width: u16, height: u16) -> String {
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
fn table_interaction_updates_the_detail_panel_when_selection_changes() {
    let mut engine = Engine::new(TableInteractionApp::default());
    engine.render_tree();

    let mut lines = Vec::new();
    collect_text(engine.tree().expect("tree should exist"), &mut lines);
    assert!(lines.join("\n").contains("Selected row: ansiq"));

    assert!(!engine.handle_input(Key::Down));
    engine.render_tree();

    let mut lines = Vec::new();
    collect_text(engine.tree().expect("tree should exist"), &mut lines);
    assert!(lines.join("\n").contains("Selected row: activity"));

    let screen = rendered_screen(&engine, 50, 10);
    assert!(screen.contains("activity"));
    assert!(!screen.contains("> activity"));
}

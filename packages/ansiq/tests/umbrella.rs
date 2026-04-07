use ansiq::prelude::*;
use ansiq::view;

#[test]
fn umbrella_prelude_imports_common_types() {
    let _color = Color::Green;
    let _style = Style::default();
    let _rect = Rect::new(0, 0, 10, 1);
    let _list_state = ListState::default();
    let _table_state = TableState::default();
    let _scrollbar_state = ScrollbarState::default();
    let _: Element<()> = view! { <Text content="hello" /> };
}

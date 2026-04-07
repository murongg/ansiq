use ansiq::prelude::*;
use ansiq::view;

#[derive(Default)]
struct DemoApp;

impl App for DemoApp {
    type Message = ();

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        view! {
            <Text content="hello ansiq" />
        }
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn minimal_app_compiles_through_umbrella_crate() {
    let mut app = DemoApp::default();
    let mut store = ansiq::core::HookStore::default();
    let mut cx = ViewCtx::new(&mut store);
    let _ = app.render(&mut cx);
}

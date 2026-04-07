use ansiq_core::{Element, ViewCtx};
use ansiq_macros::view;
use ansiq_runtime::{App, RuntimeHandle};
use ansiq_surface::ViewportPolicy;

const ITEMS: [&str; 3] = ["Overview", "Files", "Logs"];

pub const VIEWPORT_POLICY: ViewportPolicy = ViewportPolicy::ReserveFitContent { min: 5, max: 8 };

#[derive(Default)]
pub struct ListNavigationApp;

impl App for ListNavigationApp {
    type Message = ();

    fn render(&mut self, cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        let selected = cx.signal(|| 0usize);

        view! {
            <Box direction="column" gap={1}>
                <StatusBar text="> list navigation · ready" />
                <List
                    items={ITEMS.iter().map(|item| item.to_string()).collect::<Vec<_>>()}
                    selected={Some(selected.get())}
                    on_select={{
                        let selected = selected.clone();
                        move |index| {
                            selected.set(index);
                            None
                        }
                    }}
                />
                <Text content={format!("Selected: {}", ITEMS[selected.get()])} />
            </Box>
        }
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

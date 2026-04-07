use ansiq_core::{Element, TableAlignment, ViewCtx};
use ansiq_macros::view;
use ansiq_runtime::{App, RuntimeHandle};
use ansiq_surface::ViewportPolicy;

const ROWS: [(&str, &str); 3] = [
    ("ansiq", "ready"),
    ("activity", "warming"),
    ("agent", "idle"),
];

pub const VIEWPORT_POLICY: ViewportPolicy = ViewportPolicy::ReserveFitContent { min: 6, max: 10 };

#[derive(Default)]
pub struct TableInteractionApp;

impl App for TableInteractionApp {
    type Message = ();

    fn render(&mut self, cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        let selected = cx.signal(|| 0usize);

        view! {
            <Box direction="column" gap={1}>
                <StatusBar text="> table interaction · ready" />
                <Table
                    headers={vec!["Name".to_string(), "Status".to_string()]}
                    rows={ROWS
                        .iter()
                        .map(|(name, status)| vec![(*name).to_string(), (*status).to_string()])
                        .collect::<Vec<_>>()}
                    alignments={vec![TableAlignment::Left, TableAlignment::Left]}
                    selected={Some(selected.get())}
                    on_select={{
                        let selected = selected.clone();
                        move |index| {
                            selected.set(index);
                            None
                        }
                    }}
                />
                <Text content={format!("Selected row: {}", ROWS[selected.get()].0)} />
            </Box>
        }
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

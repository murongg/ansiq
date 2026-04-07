use ansiq_core::{Element, Layout, Length, ViewCtx};
use ansiq_macros::view;
use ansiq_runtime::{App, RuntimeHandle};
use ansiq_surface::ViewportPolicy;

const CONTENT: &str = "one\ntwo\nthree\nfour";

pub const VIEWPORT_POLICY: ViewportPolicy = ViewportPolicy::ReserveFitContent { min: 5, max: 8 };

#[derive(Default)]
pub struct ScrollSyncApp;

impl App for ScrollSyncApp {
    type Message = ();

    fn render(&mut self, cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        let offset = cx.signal(|| 0usize);

        view! {
            <Box direction="column" gap={1}>
                <StatusBar text="> scroll sync · ready" />
                <Box
                    direction="row"
                    gap={1}
                    layout={Layout {
                        width: Length::Fill,
                        height: Length::Fixed(2),
                    }}
                >
                    <ScrollView
                        follow_bottom={false}
                        offset={offset.get()}
                        on_scroll={{
                            let offset = offset.clone();
                            move |position| {
                                offset.set(position);
                                None
                            }
                        }}
                        layout={Layout {
                            width: Length::Fill,
                            height: Length::Fill,
                        }}
                    >
                        <Paragraph content={CONTENT} />
                    </ScrollView>
                    <Scrollbar
                        position={offset.get()}
                        content_length={4usize}
                        viewport_length={2usize}
                        on_scroll={{
                            let offset = offset.clone();
                            move |position| {
                                offset.set(position);
                                None
                            }
                        }}
                        layout={Layout {
                            width: Length::Fixed(1),
                            height: Length::Fill,
                        }}
                    />
                </Box>
                <Text content={format!("offset: {}", offset.get())} />
            </Box>
        }
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

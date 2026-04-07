use ansiq_core::{Element, ElementKind, Layout, Length, StreamingTextProps};

pub struct StreamingText<Message = ()> {
    element: Element<Message>,
}

impl<Message> StreamingText<Message> {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            element: Element::new(ElementKind::StreamingText(StreamingTextProps {
                content: content.into(),
            }))
            .with_layout(Layout {
                width: Length::Fill,
                height: Length::Fill,
            }),
        }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.element.layout = layout;
        self
    }

    pub fn build(self) -> Element<Message> {
        self.element
    }
}

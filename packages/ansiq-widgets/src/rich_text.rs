use ansiq_core::{Element, ElementKind, HistoryBlock, Layout, Length};

pub struct RichText<Message = ()> {
    element: Element<Message>,
}

impl<Message> RichText<Message> {
    pub fn new(block: HistoryBlock) -> Self {
        Self {
            element: Element::new(ElementKind::RichText(ansiq_core::RichTextProps { block }))
                .with_layout(Layout {
                    width: Length::Fill,
                    height: Length::Auto,
                }),
        }
    }

    pub fn build(self) -> Element<Message> {
        self.element
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.element.layout = layout;
        self
    }
}

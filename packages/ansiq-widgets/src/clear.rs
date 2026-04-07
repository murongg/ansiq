use ansiq_core::{ClearProps, Element, ElementKind, Layout, Length, Style};

pub struct Clear<Message = ()> {
    element: Element<Message>,
}

impl<Message> Clear<Message> {
    pub fn new() -> Self {
        Self {
            element: Element::new(ElementKind::Clear(ClearProps)).with_layout(Layout {
                width: Length::Fill,
                height: Length::Fill,
            }),
        }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.element.layout = layout;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.element.style = style;
        self
    }

    pub fn build(self) -> Element<Message> {
        self.element
    }
}

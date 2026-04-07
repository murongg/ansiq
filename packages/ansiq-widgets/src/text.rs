use ansiq_core::{Element, ElementKind, Layout, Style, TextProps};

pub struct Text<Message = ()> {
    element: Element<Message>,
}

impl<Message> Text<Message> {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            element: Element::new(ElementKind::Text(TextProps {
                content: content.into(),
            })),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.element.style = style;
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.element.layout = layout;
        self
    }

    pub fn build(self) -> Element<Message> {
        self.element
    }
}

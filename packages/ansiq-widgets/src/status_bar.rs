use ansiq_core::{Element, ElementKind, Layout, Length, StatusBarProps};

pub struct StatusBar<Message = ()> {
    element: Element<Message>,
}

impl<Message> StatusBar<Message> {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            element: Element::new(ElementKind::StatusBar(StatusBarProps {
                content: content.into(),
            }))
            .with_layout(Layout {
                width: Length::Fill,
                height: Length::Fixed(1),
            }),
        }
    }

    pub fn build(self) -> Element<Message> {
        self.element
    }
}

use ansiq_core::{Element, ElementKind, Layout, Length, PaneProps};

pub struct Pane<Message = ()> {
    element: Element<Message>,
}

impl<Message> Pane<Message> {
    pub fn new() -> Self {
        Self {
            element: Element::new(ElementKind::Pane(PaneProps { title: None })).with_layout(
                Layout {
                    width: Length::Fill,
                    height: Length::Fill,
                },
            ),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        if let ElementKind::Pane(props) = &mut self.element.kind {
            props.title = Some(title.into());
        }
        self
    }

    pub fn child(mut self, child: Element<Message>) -> Self {
        self.element.children.push(child);
        self
    }

    pub fn children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = Element<Message>>,
    {
        self.element.children.extend(children);
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

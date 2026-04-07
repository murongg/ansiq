use ansiq_core::{BoxProps, Direction, Element, ElementKind, Layout, Length, Style};

pub struct Box<Message> {
    element: Element<Message>,
}

impl<Message> Box<Message> {
    pub fn column() -> Self {
        Self::new(Direction::Column)
    }

    pub fn row() -> Self {
        Self::new(Direction::Row)
    }

    fn new(direction: Direction) -> Self {
        Self {
            element: Element::new(ElementKind::Box(BoxProps { direction, gap: 0 })).with_layout(
                Layout {
                    width: Length::Fill,
                    height: Length::Fill,
                },
            ),
        }
    }

    pub fn gap(mut self, gap: u16) -> Self {
        if let ElementKind::Box(props) = &mut self.element.kind {
            props.gap = gap;
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

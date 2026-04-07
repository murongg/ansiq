use ansiq_core::{CanvasCell, CanvasProps, Element, ElementKind, Layout, Length, Style};

pub struct Canvas<Message = ()> {
    element: Element<Message>,
}

impl<Message> Canvas<Message> {
    pub fn new() -> Self {
        Self {
            element: Element::new(ElementKind::Canvas(CanvasProps {
                width: 16,
                height: 8,
                cells: Vec::new(),
            }))
            .with_layout(Layout {
                width: Length::Fill,
                height: Length::Fixed(8),
            }),
        }
    }

    pub fn size(mut self, width: u16, height: u16) -> Self {
        if let ElementKind::Canvas(props) = &mut self.element.kind {
            props.width = width.max(1);
            props.height = height.max(1);
        }
        self
    }

    pub fn point(mut self, x: u16, y: u16, symbol: char) -> Self {
        if let ElementKind::Canvas(props) = &mut self.element.kind {
            props.cells.push(CanvasCell {
                x,
                y,
                symbol,
                style: self.element.style,
            });
        }
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.element.layout = layout;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.element.style = style;
        if let ElementKind::Canvas(props) = &mut self.element.kind {
            for cell in &mut props.cells {
                cell.style = style;
            }
        }
        self
    }

    pub fn build(self) -> Element<Message> {
        self.element
    }
}

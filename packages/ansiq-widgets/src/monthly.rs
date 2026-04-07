use ansiq_core::{Element, ElementKind, Layout, Length, MonthlyProps, Style};

pub struct Monthly<Message = ()> {
    element: Element<Message>,
}

impl<Message> Monthly<Message> {
    pub fn new() -> Self {
        Self {
            element: Element::new(ElementKind::Monthly(MonthlyProps {
                year: 2026,
                month: 1,
                selected_day: None,
            }))
            .with_layout(Layout {
                width: Length::Fill,
                height: Length::Fixed(8),
            }),
        }
    }

    pub fn year(mut self, year: i32) -> Self {
        if let ElementKind::Monthly(props) = &mut self.element.kind {
            props.year = year;
        }
        self
    }

    pub fn month(mut self, month: u8) -> Self {
        if let ElementKind::Monthly(props) = &mut self.element.kind {
            props.month = month.clamp(1, 12);
        }
        self
    }

    pub fn selected_day(mut self, day: u8) -> Self {
        if let ElementKind::Monthly(props) = &mut self.element.kind {
            props.selected_day = Some(day.max(1));
        }
        self
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

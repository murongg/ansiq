use ansiq_core::{ChartDataset, ChartProps, Element, ElementKind, Layout, Length, Style};

pub struct Chart<Message = ()> {
    element: Element<Message>,
}

impl<Message> Chart<Message> {
    pub fn new() -> Self {
        Self {
            element: Element::new(ElementKind::Chart(ChartProps {
                datasets: Vec::new(),
                min_y: None,
                max_y: None,
            }))
            .with_layout(Layout {
                width: Length::Fill,
                height: Length::Fixed(8),
            }),
        }
    }

    pub fn dataset<I>(mut self, points: I) -> Self
    where
        I: IntoIterator<Item = (i64, i64)>,
    {
        if let ElementKind::Chart(props) = &mut self.element.kind {
            props.datasets.push(ChartDataset {
                label: None,
                points: points.into_iter().collect(),
            });
        }
        self
    }

    pub fn named_dataset<I>(mut self, label: impl Into<String>, points: I) -> Self
    where
        I: IntoIterator<Item = (i64, i64)>,
    {
        if let ElementKind::Chart(props) = &mut self.element.kind {
            props.datasets.push(ChartDataset {
                label: Some(label.into()),
                points: points.into_iter().collect(),
            });
        }
        self
    }

    pub fn min_y(mut self, min_y: i64) -> Self {
        if let ElementKind::Chart(props) = &mut self.element.kind {
            props.min_y = Some(min_y);
        }
        self
    }

    pub fn max_y(mut self, max_y: i64) -> Self {
        if let ElementKind::Chart(props) = &mut self.element.kind {
            props.max_y = Some(max_y);
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

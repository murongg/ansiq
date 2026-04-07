use ansiq_core::{Bar, BarChartProps, Element, ElementKind, Layout, Length, Style};

pub struct BarChart<Message = ()> {
    element: Element<Message>,
}

impl<Message> BarChart<Message> {
    pub fn new() -> Self {
        Self {
            element: Element::new(ElementKind::BarChart(BarChartProps {
                bars: Vec::new(),
                max: None,
                bar_width: 3,
            }))
            .with_layout(Layout {
                width: Length::Fill,
                height: Length::Fixed(6),
            }),
        }
    }

    pub fn bar(mut self, label: impl Into<String>, value: u64) -> Self {
        if let ElementKind::BarChart(props) = &mut self.element.kind {
            props.bars.push(Bar {
                label: label.into(),
                value,
            });
        }
        self
    }

    pub fn bars<I, S>(mut self, bars: I) -> Self
    where
        I: IntoIterator<Item = (S, u64)>,
        S: Into<String>,
    {
        if let ElementKind::BarChart(props) = &mut self.element.kind {
            props
                .bars
                .extend(bars.into_iter().map(|(label, value)| Bar {
                    label: label.into(),
                    value,
                }));
        }
        self
    }

    pub fn max(mut self, max: u64) -> Self {
        if let ElementKind::BarChart(props) = &mut self.element.kind {
            props.max = Some(max);
        }
        self
    }

    pub fn bar_width(mut self, bar_width: u16) -> Self {
        if let ElementKind::BarChart(props) = &mut self.element.kind {
            props.bar_width = bar_width.max(1);
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

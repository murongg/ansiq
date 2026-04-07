use ansiq_core::{Element, ElementKind, Layout, Length, SparklineDirection, SparklineProps, Style};

pub struct Sparkline<Message = ()> {
    element: Element<Message>,
}

impl<Message> Sparkline<Message> {
    pub fn new() -> Self {
        Self {
            element: Element::new(ElementKind::Sparkline(SparklineProps {
                values: Vec::new(),
                max: None,
                direction: SparklineDirection::LeftToRight,
                absent_value_symbol: '·',
                absent_value_style: Style::default(),
            }))
            .with_layout(Layout {
                width: Length::Fill,
                height: Length::Fixed(1),
            }),
        }
    }

    pub fn value(mut self, value: u64) -> Self {
        if let ElementKind::Sparkline(props) = &mut self.element.kind {
            props.values.push(Some(value));
        }
        self
    }

    pub fn values<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = u64>,
    {
        if let ElementKind::Sparkline(props) = &mut self.element.kind {
            props.values.extend(values.into_iter().map(Some));
        }
        self
    }

    pub fn value_opt(mut self, value: Option<u64>) -> Self {
        if let ElementKind::Sparkline(props) = &mut self.element.kind {
            props.values.push(value);
        }
        self
    }

    pub fn values_opt<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = Option<u64>>,
    {
        if let ElementKind::Sparkline(props) = &mut self.element.kind {
            props.values.extend(values);
        }
        self
    }

    pub fn max(mut self, max: u64) -> Self {
        if let ElementKind::Sparkline(props) = &mut self.element.kind {
            props.max = Some(max);
        }
        self
    }

    pub fn direction(mut self, direction: SparklineDirection) -> Self {
        if let ElementKind::Sparkline(props) = &mut self.element.kind {
            props.direction = direction;
        }
        self
    }

    pub fn absent_symbol(mut self, symbol: char) -> Self {
        if let ElementKind::Sparkline(props) = &mut self.element.kind {
            props.absent_value_symbol = symbol;
        }
        self
    }

    pub fn absent_style(mut self, style: Style) -> Self {
        if let ElementKind::Sparkline(props) = &mut self.element.kind {
            props.absent_value_style = style;
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

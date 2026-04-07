use ansiq_core::{Element, ElementKind, Layout, Length, Line, LineGaugeProps, Style};

pub struct LineGauge<Message = ()> {
    element: Element<Message>,
}

impl<Message> Default for LineGauge<Message> {
    fn default() -> Self {
        Self {
            element: Element::new(ElementKind::LineGauge(LineGaugeProps {
                block: None,
                ratio: 0.0,
                label: None,
                filled_symbol: ansiq_core::symbols::line::HORIZONTAL.to_string(),
                unfilled_symbol: ansiq_core::symbols::line::HORIZONTAL.to_string(),
                filled_style: Style::default(),
                unfilled_style: Style::default(),
            }))
            .with_layout(Layout {
                width: Length::Fill,
                height: Length::Auto,
            }),
        }
    }
}

impl<Message> LineGauge<Message> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn block(mut self, block: crate::Block<Message>) -> Self {
        if let ElementKind::LineGauge(props) = &mut self.element.kind {
            props.block = Some(block.into_frame());
        }
        self
    }

    pub fn percent(self, percent: u16) -> Self {
        assert!(
            percent <= 100,
            "Percentage should be between 0 and 100 inclusively."
        );
        self.ratio(f64::from(percent) / 100.0)
    }

    pub fn ratio(mut self, ratio: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&ratio),
            "Ratio should be between 0 and 1 inclusively."
        );
        if let ElementKind::LineGauge(props) = &mut self.element.kind {
            props.ratio = ratio;
        }
        self
    }

    pub fn label<T>(mut self, label: T) -> Self
    where
        T: Into<Line>,
    {
        if let ElementKind::LineGauge(props) = &mut self.element.kind {
            props.label = Some(label.into());
        }
        self
    }

    pub fn line_set(mut self, set: ansiq_core::symbols::line::Set) -> Self {
        if let ElementKind::LineGauge(props) = &mut self.element.kind {
            props.filled_symbol = set.horizontal.to_string();
            props.unfilled_symbol = set.horizontal.to_string();
        }
        self
    }

    pub fn filled_symbol(mut self, symbol: impl Into<String>) -> Self {
        if let ElementKind::LineGauge(props) = &mut self.element.kind {
            props.filled_symbol = symbol.into();
        }
        self
    }

    pub fn unfilled_symbol(mut self, symbol: impl Into<String>) -> Self {
        if let ElementKind::LineGauge(props) = &mut self.element.kind {
            props.unfilled_symbol = symbol.into();
        }
        self
    }

    pub fn filled_style<S: Into<Style>>(mut self, style: S) -> Self {
        if let ElementKind::LineGauge(props) = &mut self.element.kind {
            props.filled_style = style.into();
        }
        self
    }

    pub fn unfilled_style<S: Into<Style>>(mut self, style: S) -> Self {
        if let ElementKind::LineGauge(props) = &mut self.element.kind {
            props.unfilled_style = style.into();
        }
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.element.layout = layout;
        self
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.element.style = style.into();
        self
    }

    pub fn build(self) -> Element<Message> {
        self.element
    }
}

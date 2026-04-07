use ansiq_core::{Element, ElementKind, GaugeProps, Layout, Length, Span, Style};

pub struct Gauge<Message = ()> {
    element: Element<Message>,
}

impl<Message> Default for Gauge<Message> {
    fn default() -> Self {
        Self {
            element: Element::new(ElementKind::Gauge(GaugeProps {
                block: None,
                ratio: 0.0,
                label: None,
                use_unicode: false,
                gauge_style: Style::default(),
            }))
            .with_layout(Layout {
                width: Length::Fill,
                height: Length::Auto,
            }),
        }
    }
}

impl<Message> Gauge<Message> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn block(mut self, block: crate::Block<Message>) -> Self {
        if let ElementKind::Gauge(props) = &mut self.element.kind {
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
        if let ElementKind::Gauge(props) = &mut self.element.kind {
            props.ratio = ratio;
        }
        self
    }

    pub fn label<T>(mut self, label: T) -> Self
    where
        T: Into<Span>,
    {
        if let ElementKind::Gauge(props) = &mut self.element.kind {
            props.label = Some(label.into());
        }
        self
    }

    pub fn use_unicode(mut self, use_unicode: bool) -> Self {
        if let ElementKind::Gauge(props) = &mut self.element.kind {
            props.use_unicode = use_unicode;
        }
        self
    }

    pub fn gauge_style<S: Into<Style>>(mut self, style: S) -> Self {
        if let ElementKind::Gauge(props) = &mut self.element.kind {
            props.gauge_style = style.into();
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

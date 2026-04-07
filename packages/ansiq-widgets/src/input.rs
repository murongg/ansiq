use ansiq_core::{
    ChangeHandler, Element, ElementKind, InputProps, Layout, Length, Style, SubmitHandler,
};

pub struct Input<Message = ()> {
    value: String,
    placeholder: String,
    on_change: Option<ChangeHandler>,
    on_submit: Option<SubmitHandler<Message>>,
    layout: Layout,
    style: Style,
    focusable: bool,
}

impl Input<()> {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            placeholder: String::new(),
            on_change: None,
            on_submit: None,
            layout: Layout {
                width: Length::Fill,
                height: Length::Fixed(3),
            },
            style: Style::default(),
            focusable: true,
        }
    }
}

impl<Message> Input<Message> {
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: FnMut(String) + 'static,
    {
        self.on_change = Some(std::boxed::Box::new(handler));
        self
    }

    pub fn on_submit<NextMessage, F>(self, handler: F) -> Input<NextMessage>
    where
        F: FnMut(String) -> Option<NextMessage> + 'static,
    {
        Input {
            value: self.value,
            placeholder: self.placeholder,
            on_change: self.on_change,
            on_submit: Some(std::boxed::Box::new(handler)),
            layout: self.layout,
            style: self.style,
            focusable: self.focusable,
        }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn build(self) -> Element<Message> {
        let Input {
            value,
            placeholder,
            on_change,
            on_submit,
            layout,
            style,
            focusable,
        } = self;
        let cursor = value.chars().count();

        Element::new(ElementKind::Input(InputProps {
            value,
            placeholder,
            on_change,
            on_submit,
            cursor,
        }))
        .with_layout(layout)
        .with_style(style)
        .with_focusable(focusable)
    }
}

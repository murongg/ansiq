use ansiq_core::{ChangeHandler, Element, Layout, Length, Style, SubmitHandler};

use crate::{Box, Input, Text};

pub struct ComposerBar<Message = ()> {
    value: String,
    placeholder: String,
    meta: Option<String>,
    on_change: Option<ChangeHandler>,
    on_submit: Option<SubmitHandler<Message>>,
    input_style: Style,
    meta_style: Style,
}

impl ComposerBar<()> {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            placeholder: String::new(),
            meta: None,
            on_change: None,
            on_submit: None,
            input_style: Style::default(),
            meta_style: Style::default(),
        }
    }
}

impl<Message: 'static> ComposerBar<Message> {
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn meta(mut self, meta: impl Into<String>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: FnMut(String) + 'static,
    {
        self.on_change = Some(std::boxed::Box::new(handler));
        self
    }

    pub fn on_submit<NextMessage, F>(self, handler: F) -> ComposerBar<NextMessage>
    where
        F: FnMut(String) -> Option<NextMessage> + 'static,
    {
        ComposerBar {
            value: self.value,
            placeholder: self.placeholder,
            meta: self.meta,
            on_change: self.on_change,
            on_submit: Some(std::boxed::Box::new(handler)),
            input_style: self.input_style,
            meta_style: self.meta_style,
        }
    }

    pub fn input_style(mut self, style: Style) -> Self {
        self.input_style = style;
        self
    }

    pub fn meta_style(mut self, style: Style) -> Self {
        self.meta_style = style;
        self
    }

    pub fn build(self) -> Element<Message> {
        let mut column = Box::column().gap(0).layout(Layout {
            width: Length::Fill,
            height: Length::Auto,
        });

        let input = Input::new()
            .value(self.value)
            .placeholder(self.placeholder)
            .style(self.input_style);
        let input = if let Some(handler) = self.on_change {
            input.on_change(handler)
        } else {
            input
        };
        let input = if let Some(handler) = self.on_submit {
            input.on_submit(handler)
        } else {
            input.on_submit(|_| None::<Message>)
        };
        column = column.child(input.build());

        if let Some(meta) = self.meta {
            column = column.child(Text::new(meta).style(self.meta_style).build());
        }

        column.build()
    }
}

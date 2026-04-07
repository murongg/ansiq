use ansiq_core::{Element, Layout, Length, Style};

use crate::Box;

pub struct BottomPane<Message = ()> {
    composer: Option<Element<Message>>,
    footer: Option<Element<Message>>,
    style: Style,
    gap: u16,
}

impl<Message> BottomPane<Message> {
    pub fn new() -> Self {
        Self {
            composer: None,
            footer: None,
            style: Style::default(),
            gap: 0,
        }
    }

    pub fn composer(mut self, composer: Element<Message>) -> Self {
        self.composer = Some(composer);
        self
    }

    pub fn footer(mut self, footer: Element<Message>) -> Self {
        self.footer = Some(footer);
        self
    }

    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn build(self) -> Element<Message> {
        let mut column = Box::column()
            .gap(self.gap)
            .style(self.style)
            .layout(Layout {
                width: Length::Fill,
                height: Length::Auto,
            });

        if let Some(composer) = self.composer {
            column = column.child(composer);
        }

        if let Some(footer) = self.footer {
            column = column.child(footer);
        }

        column.build()
    }
}

use ansiq_core::{Element, Layout, Length, Style};

use crate::{Box, Pane, StatusBar, Text};

pub struct SessionHeader<Message = ()> {
    status: Option<String>,
    title: Option<String>,
    meta_lines: Vec<String>,
    gap: u16,
    pane_width: u16,
    status_style: Style,
    pane_style: Style,
    title_style: Style,
    meta_style: Style,
    marker: std::marker::PhantomData<Message>,
}

impl<Message> SessionHeader<Message> {
    pub fn new() -> Self {
        Self {
            status: None,
            title: None,
            meta_lines: Vec::new(),
            gap: 1,
            pane_width: 56,
            status_style: Style::default(),
            pane_style: Style::default(),
            title_style: Style::default(),
            meta_style: Style::default(),
            marker: std::marker::PhantomData,
        }
    }
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn meta_line(mut self, line: impl Into<String>) -> Self {
        self.meta_lines.push(line.into());
        self
    }

    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    pub fn pane_width(mut self, pane_width: u16) -> Self {
        self.pane_width = pane_width;
        self
    }

    pub fn status_style(mut self, style: Style) -> Self {
        self.status_style = style;
        self
    }

    pub fn pane_style(mut self, style: Style) -> Self {
        self.pane_style = style;
        self
    }

    pub fn title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    pub fn meta_style(mut self, style: Style) -> Self {
        self.meta_style = style;
        self
    }

    pub fn build(self) -> Element<Message> {
        let mut header = Box::column().gap(self.gap).layout(Layout {
            width: Length::Fill,
            height: Length::Auto,
        });

        if let Some(status) = self.status {
            header = header.child(StatusBar::new(status).build().with_style(self.status_style));
        }

        let mut banner = Box::column().gap(0).style(self.pane_style);
        if let Some(title) = self.title {
            banner = banner.child(Text::new(title).style(self.title_style).build());
        }
        for line in self.meta_lines {
            banner = banner.child(Text::new(line).style(self.meta_style).build());
        }

        let pane = Pane::new()
            .layout(Layout {
                width: Length::Fixed(self.pane_width),
                height: Length::Auto,
            })
            .child(banner.build())
            .build()
            .with_style(self.pane_style);

        header.child(pane).build()
    }
}

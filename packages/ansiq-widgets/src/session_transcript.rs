use ansiq_core::{Element, Layout, Length, Style, TranscriptEntry, TranscriptSession};

use crate::{Box, TranscriptView};

pub struct SessionTranscript<Message = ()> {
    intro: Option<Element<Message>>,
    active: Option<Element<Message>>,
    entries: Vec<TranscriptEntry>,
    empty: Option<Element<Message>>,
    session_started: bool,
    style: Style,
    gap: u16,
}

impl<Message> SessionTranscript<Message> {
    pub fn new() -> Self {
        Self {
            intro: None,
            active: None,
            entries: Vec::new(),
            empty: None,
            session_started: false,
            style: Style::default(),
            gap: 1,
        }
    }

    pub fn intro(mut self, intro: Element<Message>) -> Self {
        self.intro = Some(intro);
        self
    }

    pub fn active(mut self, active: Element<Message>) -> Self {
        self.active = Some(active);
        self
    }

    pub fn entries(mut self, entries: impl IntoIterator<Item = TranscriptEntry>) -> Self {
        self.entries = entries.into_iter().collect();
        self
    }

    pub fn session(mut self, session: TranscriptSession) -> Self {
        self.session_started = session.started();
        self.entries = session.entries().to_vec();
        self
    }

    pub fn empty(mut self, empty: Element<Message>) -> Self {
        self.empty = Some(empty);
        self
    }

    pub fn session_started(mut self, session_started: bool) -> Self {
        self.session_started = session_started;
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

        if !self.session_started {
            if let Some(intro) = self.intro {
                column = column.child(intro);
            }
        } else if !self.entries.is_empty() {
            column = column.child(TranscriptView::new(self.entries).build());
        } else if let Some(active) = self.active {
            column = column.child(active);
        } else if let Some(empty) = self.empty {
            column = column.child(empty);
        }

        column.build()
    }
}

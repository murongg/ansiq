use ansiq_core::{Element, Layout, Length, TranscriptEntry, transcript_block};

use crate::RichText;

pub struct TranscriptView<Message = ()> {
    entries: Vec<TranscriptEntry>,
    layout: Layout,
    marker: std::marker::PhantomData<Message>,
}

impl<Message> TranscriptView<Message> {
    pub fn new(entries: impl IntoIterator<Item = TranscriptEntry>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
            layout: Layout {
                width: Length::Fill,
                height: Length::Auto,
            },
            marker: std::marker::PhantomData,
        }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn build(self) -> Element<Message> {
        RichText::new(transcript_block(&self.entries))
            .layout(self.layout)
            .build()
    }
}

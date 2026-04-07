use crate::{Color, HistoryBlock, HistoryLine, HistoryRun, Style};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TranscriptRole {
    User,
    Assistant,
    Status,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranscriptEntry {
    pub role: TranscriptRole,
    pub content: String,
}

impl TranscriptEntry {
    pub fn new(role: TranscriptRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new(TranscriptRole::User, content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(TranscriptRole::Assistant, content)
    }

    pub fn status(content: impl Into<String>) -> Self {
        Self::new(TranscriptRole::Status, content)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TranscriptSession {
    started: bool,
    entries: Vec<TranscriptEntry>,
}

impl TranscriptSession {
    pub fn started(&self) -> bool {
        self.started
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[TranscriptEntry] {
        &self.entries
    }

    pub fn begin_turn(&mut self, prompt: impl Into<String>) -> Option<HistoryBlock> {
        let committed = (!self.entries.is_empty()).then(|| transcript_block(&self.entries));
        self.started = true;
        self.entries = vec![
            TranscriptEntry::user(prompt),
            TranscriptEntry::assistant(String::new()),
        ];
        committed
    }

    pub fn append_assistant(&mut self, chunk: &str) {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .rev()
            .find(|entry| matches!(entry.role, TranscriptRole::Assistant))
        {
            entry.content.push_str(chunk);
        }
    }
}

pub fn transcript_block(entries: &[TranscriptEntry]) -> HistoryBlock {
    let mut lines = Vec::new();

    for entry in entries {
        let entry_lines: Vec<&str> = entry.content.lines().collect();
        if entry_lines.is_empty() {
            lines.push(transcript_line(entry.role, ""));
            continue;
        }

        for (index, line) in entry_lines.into_iter().enumerate() {
            lines.push(transcript_line_with_prefix(entry.role, line, index == 0));
        }
    }

    HistoryBlock { lines }
}

fn transcript_line(role: TranscriptRole, content: &str) -> HistoryLine {
    transcript_line_with_prefix(role, content, true)
}

fn transcript_line_with_prefix(
    role: TranscriptRole,
    content: &str,
    include_prefix: bool,
) -> HistoryLine {
    let mut runs = Vec::new();
    let prefix = match (role, include_prefix) {
        (TranscriptRole::User, true) => Some(("you  ", user_style())),
        (TranscriptRole::Assistant, true) => Some(("assistant  ", assistant_style())),
        (TranscriptRole::Status, _) => None,
        _ => None,
    };

    if let Some((label, style)) = prefix {
        runs.push(HistoryRun {
            text: label.to_string(),
            style,
        });
    }

    runs.push(HistoryRun {
        text: content.to_string(),
        style: content_style(role),
    });

    HistoryLine { runs }
}

fn user_style() -> Style {
    Style::default().fg(Color::Grey).bold(true)
}

fn assistant_style() -> Style {
    Style::default().fg(Color::Grey).bold(true)
}

fn content_style(role: TranscriptRole) -> Style {
    match role {
        TranscriptRole::Status => Style::default().fg(Color::DarkGrey),
        TranscriptRole::User | TranscriptRole::Assistant => Style::default().fg(Color::Grey),
    }
}

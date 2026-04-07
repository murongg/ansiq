use crate::Style;
use unicode_width::UnicodeWidthChar;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryRun {
    pub text: String,
    pub style: Style,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryLine {
    pub runs: Vec<HistoryRun>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryBlock {
    pub lines: Vec<HistoryLine>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HistoryEntry {
    Text(String),
    Block(HistoryBlock),
}

pub fn history_block_from_text(content: &str, width: u16) -> HistoryBlock {
    if width == 0 {
        return HistoryBlock { lines: Vec::new() };
    }

    let mut lines = Vec::new();

    for raw_line in content.split('\n') {
        if raw_line.is_empty() {
            lines.push(HistoryLine { runs: Vec::new() });
            continue;
        }

        let mut current = String::new();
        let mut current_width = 0u16;

        for ch in raw_line.chars() {
            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
            let char_width = char_width.max(1);
            if current_width.saturating_add(char_width) > width && !current.is_empty() {
                lines.push(HistoryLine {
                    runs: vec![HistoryRun {
                        text: std::mem::take(&mut current),
                        style: Style::default(),
                    }],
                });
                current_width = 0;
            }

            current.push(ch);
            current_width = current_width.saturating_add(char_width);
        }

        lines.push(HistoryLine {
            runs: vec![HistoryRun {
                text: current,
                style: Style::default(),
            }],
        });
    }

    if lines.is_empty() {
        lines.push(HistoryLine { runs: Vec::new() });
    }

    HistoryBlock { lines }
}

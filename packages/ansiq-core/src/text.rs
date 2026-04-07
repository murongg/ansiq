use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{Alignment, Style, patch_style};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StyledChunk {
    pub text: String,
    pub style: Style,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StyledLine {
    pub chunks: Vec<StyledChunk>,
    pub alignment: Alignment,
    pub width: u16,
}

#[derive(Clone)]
struct StyledToken {
    text: String,
    style: Style,
    is_whitespace: bool,
    width: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Span {
    pub content: String,
    pub style: Style,
}

impl Span {
    pub fn raw(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: Style::default(),
        }
    }

    pub fn styled(content: impl Into<String>, style: Style) -> Self {
        Self {
            content: content.into(),
            style,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn width(&self) -> usize {
        UnicodeWidthStr::width(self.content.as_str())
    }
}

impl From<&str> for Span {
    fn from(value: &str) -> Self {
        Self::raw(value)
    }
}

impl From<String> for Span {
    fn from(value: String) -> Self {
        Self::raw(value)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Line {
    pub spans: Vec<Span>,
    pub alignment: Option<Alignment>,
}

impl Line {
    pub fn raw(content: impl Into<String>) -> Self {
        Self {
            spans: vec![Span::raw(content)],
            alignment: None,
        }
    }

    pub fn styled(content: impl Into<String>, style: Style) -> Self {
        Self {
            spans: vec![Span::styled(content, style)],
            alignment: None,
        }
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    pub fn left_aligned(self) -> Self {
        self.alignment(Alignment::Left)
    }

    pub fn centered(self) -> Self {
        self.alignment(Alignment::Center)
    }

    pub fn right_aligned(self) -> Self {
        self.alignment(Alignment::Right)
    }

    pub fn width(&self) -> usize {
        self.spans.iter().map(Span::width).sum()
    }

    pub const fn height(&self) -> usize {
        1
    }

    pub fn plain(&self) -> String {
        self.spans
            .iter()
            .map(|span| span.content.as_str())
            .collect::<String>()
    }
}

impl From<&str> for Line {
    fn from(value: &str) -> Self {
        Self::raw(value)
    }
}

impl From<String> for Line {
    fn from(value: String) -> Self {
        Self::raw(value)
    }
}

impl From<Span> for Line {
    fn from(value: Span) -> Self {
        Self {
            spans: vec![value],
            alignment: None,
        }
    }
}

impl From<Vec<Span>> for Line {
    fn from(value: Vec<Span>) -> Self {
        Self {
            spans: value,
            alignment: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Text {
    pub lines: Vec<Line>,
    pub alignment: Option<Alignment>,
}

impl Text {
    pub fn raw(content: impl Into<String>) -> Self {
        Self::from(content.into())
    }

    pub fn styled(content: impl Into<String>, style: Style) -> Self {
        Self::from(Line::styled(content, style))
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    pub fn left_aligned(self) -> Self {
        self.alignment(Alignment::Left)
    }

    pub fn centered(self) -> Self {
        self.alignment(Alignment::Center)
    }

    pub fn right_aligned(self) -> Self {
        self.alignment(Alignment::Right)
    }

    pub fn height(&self) -> usize {
        self.lines.len().max(1)
    }

    pub fn width(&self) -> usize {
        self.lines.iter().map(Line::width).max().unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.lines.iter().all(|line| line.spans.is_empty())
    }

    pub fn plain(&self) -> String {
        self.lines
            .iter()
            .map(Line::plain)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl From<&str> for Text {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}

impl From<String> for Text {
    fn from(value: String) -> Self {
        let lines = if value.is_empty() {
            vec![Line::default()]
        } else {
            value.split('\n').map(Line::raw).collect()
        };

        Self {
            lines,
            alignment: None,
        }
    }
}

impl From<Span> for Text {
    fn from(value: Span) -> Self {
        Self::from(Line::from(value))
    }
}

impl From<Line> for Text {
    fn from(value: Line) -> Self {
        let alignment = value.alignment;
        Self {
            lines: vec![value],
            alignment,
        }
    }
}

impl From<Vec<Line>> for Text {
    fn from(value: Vec<Line>) -> Self {
        let alignment = value.iter().find_map(|line| line.alignment);
        Self {
            lines: if value.is_empty() {
                vec![Line::default()]
            } else {
                value
            },
            alignment,
        }
    }
}

pub fn display_width(text: &str) -> u16 {
    text.chars()
        .map(|ch| UnicodeWidthChar::width(ch).unwrap_or(0) as u16)
        .sum()
}

pub fn display_width_prefix(text: &str, cursor: usize) -> u16 {
    text.chars()
        .take(cursor)
        .map(|ch| UnicodeWidthChar::width(ch).unwrap_or(0) as u16)
        .sum()
}

pub fn clip_to_width(content: &str, width: u16) -> String {
    if width == 0 {
        return String::new();
    }

    let mut clipped = String::new();
    let mut used = 0u16;

    for ch in content.chars() {
        let char_width = (UnicodeWidthChar::width(ch).unwrap_or(0) as u16).max(1);
        if used.saturating_add(char_width) > width {
            break;
        }
        clipped.push(ch);
        used = used.saturating_add(char_width);
    }

    clipped
}

pub fn wrap_plain_lines(content: &str, width: u16, trim_leading: bool) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }

    let mut lines = Vec::new();

    for raw_line in content.split('\n') {
        if raw_line.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current = String::new();
        let mut current_width = 0u16;

        for ch in raw_line.chars() {
            let char_width = (UnicodeWidthChar::width(ch).unwrap_or(0) as u16).max(1);
            if current_width.saturating_add(char_width) > width && !current.is_empty() {
                lines.push(current);
                current = String::new();
                current_width = 0;
                if trim_leading && ch.is_whitespace() {
                    continue;
                }
            }

            current.push(ch);
            current_width = current_width.saturating_add(char_width);
        }

        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

pub fn styled_lines_from_text(
    text: &Text,
    base_style: Style,
    fallback_alignment: Alignment,
) -> Vec<StyledLine> {
    let alignment = text.alignment.unwrap_or(fallback_alignment);
    let mut lines: Vec<StyledLine> = text
        .lines
        .iter()
        .map(|line| {
            let chunks: Vec<StyledChunk> = line
                .spans
                .iter()
                .map(|span| StyledChunk {
                    text: span.content.clone(),
                    style: patch_style(base_style, span.style),
                })
                .collect();
            StyledLine {
                width: chunks
                    .iter()
                    .map(|chunk| UnicodeWidthStr::width(chunk.text.as_str()) as u16)
                    .sum(),
                chunks,
                alignment: line.alignment.unwrap_or(alignment),
            }
        })
        .collect();

    if lines.is_empty() {
        lines.push(StyledLine {
            chunks: Vec::new(),
            alignment,
            width: 0,
        });
    }

    lines
}

pub fn styled_line_from_line(line: &Line, base_style: Style) -> StyledLine {
    styled_lines_from_text(&Text::from(line.clone()), base_style, Alignment::Left)
        .into_iter()
        .next()
        .unwrap_or(StyledLine {
            chunks: Vec::new(),
            alignment: Alignment::Left,
            width: 0,
        })
}

pub fn styled_line_from_span(span: &Span, base_style: Style) -> StyledLine {
    styled_line_from_line(&Line::from(span.clone()), base_style)
}

pub fn wrap_styled_lines(lines: &[StyledLine], width: u16, trim: bool) -> Vec<StyledLine> {
    if width == 0 {
        return Vec::new();
    }

    let mut wrapped = Vec::new();

    for line in lines {
        let mut current = StyledLine {
            chunks: Vec::new(),
            alignment: line.alignment,
            width: 0,
        };
        for token in styled_tokens_from_line(line) {
            if token.is_whitespace && trim && current.width == 0 {
                continue;
            }

            if token.width <= width {
                if current.width.saturating_add(token.width) > width && current.width > 0 {
                    wrapped.push(current);
                    current = StyledLine {
                        chunks: Vec::new(),
                        alignment: line.alignment,
                        width: 0,
                    };
                    if token.is_whitespace && trim {
                        continue;
                    }
                }

                append_token(&mut current, &token);
                continue;
            }

            let mut token_text = String::new();
            let mut token_width = 0u16;
            for ch in token.text.chars() {
                let char_width = (UnicodeWidthChar::width(ch).unwrap_or(0) as u16).max(1);

                if token.is_whitespace && trim && current.width == 0 {
                    continue;
                }

                if current
                    .width
                    .saturating_add(token_width)
                    .saturating_add(char_width)
                    > width
                    && (current.width > 0 || token_width > 0)
                {
                    if !token_text.is_empty() {
                        current.width = current.width.saturating_add(token_width);
                        push_chunk(&mut current.chunks, token_text.clone(), token.style);
                        token_text.clear();
                        token_width = 0;
                    }

                    wrapped.push(current);
                    current = StyledLine {
                        chunks: Vec::new(),
                        alignment: line.alignment,
                        width: 0,
                    };

                    if token.is_whitespace && trim {
                        continue;
                    }
                }

                token_text.push(ch);
                token_width = token_width.saturating_add(char_width);
            }

            if !token_text.is_empty() {
                current.width = current.width.saturating_add(token_width);
                push_chunk(&mut current.chunks, token_text, token.style);
            }
        }

        wrapped.push(current);
    }

    wrapped
}

fn styled_tokens_from_line(line: &StyledLine) -> Vec<StyledToken> {
    let mut tokens = Vec::new();

    for chunk in &line.chunks {
        let mut token = String::new();
        let mut token_is_whitespace = None;
        let mut token_width = 0u16;

        for ch in chunk.text.chars() {
            let is_whitespace = ch.is_whitespace();
            if token_is_whitespace.is_some() && token_is_whitespace != Some(is_whitespace) {
                tokens.push(StyledToken {
                    text: token.clone(),
                    style: chunk.style,
                    is_whitespace: token_is_whitespace.unwrap_or(false),
                    width: token_width,
                });
                token.clear();
                token_width = 0;
            }

            token_is_whitespace = Some(is_whitespace);
            token.push(ch);
            token_width = token_width
                .saturating_add((UnicodeWidthChar::width(ch).unwrap_or(0) as u16).max(1));
        }

        if !token.is_empty() {
            tokens.push(StyledToken {
                text: token,
                style: chunk.style,
                is_whitespace: token_is_whitespace.unwrap_or(false),
                width: token_width,
            });
        }
    }

    tokens
}

fn append_token(target: &mut StyledLine, token: &StyledToken) {
    target.width = target.width.saturating_add(token.width);
    push_chunk(&mut target.chunks, token.text.clone(), token.style);
}

fn push_chunk(chunks: &mut Vec<StyledChunk>, text: String, style: Style) {
    if text.is_empty() {
        return;
    }

    if let Some(last) = chunks.last_mut()
        && last.style == style
    {
        last.text.push_str(&text);
        return;
    }

    chunks.push(StyledChunk { text, style });
}

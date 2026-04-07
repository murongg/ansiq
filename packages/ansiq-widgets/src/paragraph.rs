use ansiq_core::{Alignment, Element, ElementKind, Layout, ParagraphProps, Style, Text, Wrap};
use unicode_width::UnicodeWidthChar;

use crate::Block;

pub struct Paragraph<Message = ()> {
    element: Element<Message>,
}

impl<Message> Paragraph<Message> {
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Text>,
    {
        let content = content.into();
        let alignment = content.alignment.unwrap_or(Alignment::Left);
        Self {
            element: Element::new(ElementKind::Paragraph(ParagraphProps {
                content,
                block: None,
                alignment,
                wrap: None,
                scroll_x: 0,
                scroll_y: 0,
            })),
        }
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        if let ElementKind::Paragraph(props) = &mut self.element.kind {
            props.alignment = alignment;
        }
        self
    }

    pub fn wrap(mut self, wrap: Wrap) -> Self {
        if let ElementKind::Paragraph(props) = &mut self.element.kind {
            props.wrap = Some(wrap);
        }
        self
    }

    pub fn block(mut self, block: Block<Message>) -> Self {
        if let ElementKind::Paragraph(props) = &mut self.element.kind {
            props.block = Some(block.into_frame());
        }
        self
    }

    pub fn scroll(mut self, offset: (u16, u16)) -> Self {
        if let ElementKind::Paragraph(props) = &mut self.element.kind {
            props.scroll_y = offset.0;
            props.scroll_x = offset.1;
        }
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

    pub fn line_count(&self, width: u16) -> usize {
        if width < 1 {
            return 0;
        }

        let props = self.props();
        let (top, bottom) = props
            .block
            .as_ref()
            .map(block_vertical_space)
            .unwrap_or_default();

        let count = if let Some(wrap) = props.wrap {
            props
                .content
                .lines
                .iter()
                .map(|line| wrapped_line_count(&line.plain(), width, wrap.trim))
                .sum::<usize>()
                .max(1)
        } else {
            props.content.height()
        };

        count
            .saturating_add(top as usize)
            .saturating_add(bottom as usize)
    }

    pub fn line_width(&self) -> usize {
        let props = self.props();
        let width = props
            .content
            .lines
            .iter()
            .map(ansiq_core::Line::width)
            .max()
            .unwrap_or_default();
        let (left, right) = props
            .block
            .as_ref()
            .map(block_horizontal_space)
            .unwrap_or_default();

        width
            .saturating_add(left as usize)
            .saturating_add(right as usize)
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.element.style = style.into();
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.element.layout = layout;
        self
    }

    pub fn build(self) -> Element<Message> {
        self.element
    }

    fn props(&self) -> &ParagraphProps {
        let ElementKind::Paragraph(props) = &self.element.kind else {
            unreachable!("Paragraph widgets always store paragraph props")
        };
        props
    }
}

fn block_horizontal_space(block: &ansiq_core::BlockFrame) -> (u16, u16) {
    let left = block.props.padding.left.saturating_add(u16::from(
        block.props.borders.contains(ansiq_core::Borders::LEFT),
    ));
    let right = block.props.padding.right.saturating_add(u16::from(
        block.props.borders.contains(ansiq_core::Borders::RIGHT),
    ));
    (left, right)
}

fn block_vertical_space(block: &ansiq_core::BlockFrame) -> (u16, u16) {
    let has_top = block.props.borders.contains(ansiq_core::Borders::TOP)
        || block
            .props
            .has_title_at_position(ansiq_core::TitlePosition::Top);
    let has_bottom = block.props.borders.contains(ansiq_core::Borders::BOTTOM)
        || block
            .props
            .has_title_at_position(ansiq_core::TitlePosition::Bottom);
    let top = block.props.padding.top.saturating_add(u16::from(has_top));
    let bottom = block
        .props
        .padding
        .bottom
        .saturating_add(u16::from(has_bottom));
    (top, bottom)
}

fn wrapped_line_count(content: &str, width: u16, trim: bool) -> usize {
    if content.is_empty() {
        return 1;
    }

    let mut count = 1usize;
    let mut current_width = 0u16;
    let mut token = String::new();
    let mut token_is_whitespace = None;

    let flush_token = |token: &mut String,
                       token_is_whitespace: Option<bool>,
                       current_width: &mut u16,
                       count: &mut usize| {
        if token.is_empty() {
            return;
        }

        let is_whitespace = token_is_whitespace.unwrap_or(false);
        let token_width = token
            .chars()
            .map(|ch| UnicodeWidthChar::width(ch).unwrap_or(0) as u16)
            .map(|char_width: u16| char_width.max(1))
            .sum::<u16>();

        if is_whitespace {
            if trim && *current_width == 0 {
                token.clear();
                return;
            }
            if current_width.saturating_add(token_width) > width && *current_width > 0 {
                *count = (*count).saturating_add(1);
                *current_width = 0;
                if trim {
                    token.clear();
                    return;
                }
            }
            *current_width = current_width.saturating_add(token_width);
            token.clear();
            return;
        }

        if token_width > width {
            for ch in token.chars() {
                let char_width = (UnicodeWidthChar::width(ch).unwrap_or(0) as u16).max(1);
                if current_width.saturating_add(char_width) > width && *current_width > 0 {
                    *count = (*count).saturating_add(1);
                    *current_width = 0;
                }
                *current_width = current_width.saturating_add(char_width);
            }
            token.clear();
            return;
        }

        if current_width.saturating_add(token_width) > width && *current_width > 0 {
            *count = (*count).saturating_add(1);
            *current_width = 0;
        }
        *current_width = current_width.saturating_add(token_width);
        token.clear();
    };

    for ch in content.chars() {
        let is_whitespace = ch.is_whitespace();
        if token_is_whitespace.is_some() && token_is_whitespace != Some(is_whitespace) {
            flush_token(
                &mut token,
                token_is_whitespace,
                &mut current_width,
                &mut count,
            );
        }
        token_is_whitespace = Some(is_whitespace);
        token.push(ch);
    }

    flush_token(
        &mut token,
        token_is_whitespace,
        &mut current_width,
        &mut count,
    );

    count
}

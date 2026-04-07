use ansiq_core::{
    Alignment, ElementKind, HistoryBlock, Node, ParagraphProps, Rect, Style, StyledLine, Text,
    display_width, patch_style, styled_lines_from_text, wrap_plain_lines, wrap_styled_lines,
};
use ansiq_render::FrameBuffer;

use crate::draw_border::{block_inner_rect, draw_block_frame};

pub(crate) fn draw_text(buffer: &mut FrameBuffer, rect: Rect, content: &str, style: Style) {
    let lines = wrap_plain_lines(content, rect.width, false);

    for (index, line) in lines.into_iter().enumerate() {
        let y = rect.y.saturating_add(index as u16);
        if y >= rect.bottom() {
            break;
        }

        draw_console_line(buffer, rect, y.saturating_sub(rect.y), &line, style);
    }
}

pub(crate) fn draw_paragraph(
    buffer: &mut FrameBuffer,
    rect: Rect,
    props: &ParagraphProps,
    style: Style,
) {
    let (inner_rect, text_style) = if let Some(block) = &props.block {
        let block_style = patch_style(style, block.style);
        draw_block_frame(buffer, rect, block, block_style);
        (block_inner_rect(rect, block), block_style)
    } else {
        (rect, style)
    };

    let lines = paragraph_lines(props, inner_rect.width, text_style);
    let visible = inner_rect.height as usize;
    let start = props.scroll_y as usize;

    for (index, line) in lines.into_iter().skip(start).take(visible).enumerate() {
        draw_styled_line(buffer, inner_rect, index as u16, &line, props.scroll_x);
    }
}

pub(crate) fn draw_rich_text(buffer: &mut FrameBuffer, rect: Rect, block: &HistoryBlock) {
    for (line_index, line) in block.lines.iter().enumerate() {
        let y = rect.y.saturating_add(line_index as u16);
        if y >= rect.bottom() {
            break;
        }

        let mut cursor_x = rect.x;
        for run in &line.runs {
            if cursor_x >= rect.right() {
                break;
            }

            let run_rect = Rect::new(cursor_x, y, rect.right().saturating_sub(cursor_x), 1);
            buffer.write_clipped(run_rect, 0, 0, &run.text, run.style);
            cursor_x = cursor_x.saturating_add(display_width(&run.text));
        }
    }
}

pub(crate) fn draw_scroll_text(
    buffer: &mut FrameBuffer,
    rect: Rect,
    content: &str,
    style: Style,
    follow_bottom: bool,
    offset: Option<usize>,
) {
    let lines = wrap_plain_lines(content, rect.width, false);
    let visible = rect.height as usize;
    let max_start = lines.len().saturating_sub(visible);
    let start = match offset {
        Some(offset) => offset.min(max_start),
        None if follow_bottom => max_start,
        None => 0,
    };

    for (index, line) in lines.into_iter().skip(start).take(visible).enumerate() {
        let y = rect.y.saturating_add(index as u16);
        draw_console_line(buffer, rect, y.saturating_sub(rect.y), &line, style);
    }
}

pub(crate) fn text_content<Message>(node: &Node<Message>) -> Option<(String, Style)> {
    match &node.element.kind {
        ElementKind::StreamingText(props) => Some((props.content.clone(), node.element.style)),
        ElementKind::Text(props) => Some((props.content.clone(), node.element.style)),
        ElementKind::Paragraph(props) => Some((plain_text(&props.content), node.element.style)),
        _ => None,
    }
}

pub(crate) fn plain_text(text: &Text) -> String {
    text.lines
        .iter()
        .map(|line| line.plain())
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn draw_console_line(
    buffer: &mut FrameBuffer,
    rect: Rect,
    row: u16,
    line: &str,
    style: Style,
) {
    buffer.write_clipped(rect, 0, row, line, style);
}

pub(crate) fn paragraph_lines(props: &ParagraphProps, width: u16, style: Style) -> Vec<StyledLine> {
    let lines = styled_lines_from_text(&props.content, style, props.alignment);
    if let Some(wrap) = props.wrap {
        wrap_styled_lines(&lines, width, wrap.trim)
    } else {
        lines
    }
}

pub(crate) fn append_styled_line(target: &mut StyledLine, segment: &StyledLine) {
    target.width = target.width.saturating_add(segment.width);
    target.chunks.extend(segment.chunks.clone());
}

pub(crate) fn draw_styled_line(
    buffer: &mut FrameBuffer,
    rect: Rect,
    row: u16,
    line: &StyledLine,
    scroll_x: u16,
) {
    if rect.width == 0 || row >= rect.height {
        return;
    }

    let visible_width = rect.width;
    let padding = visible_width.saturating_sub(line.width);
    let aligned_offset = match line.alignment {
        Alignment::Left => 0,
        Alignment::Center => padding / 2,
        Alignment::Right => padding,
    };

    let mut skipped = 0u16;
    let mut cursor_x = rect.x.saturating_add(aligned_offset);
    let max_x = rect.right();

    for chunk in &line.chunks {
        for ch in chunk.text.chars() {
            let char_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
            let char_width = char_width.max(1);

            if skipped.saturating_add(char_width) <= scroll_x {
                skipped = skipped.saturating_add(char_width);
                continue;
            }

            if cursor_x.saturating_add(char_width) > max_x {
                return;
            }

            buffer.set(
                cursor_x,
                rect.y.saturating_add(row),
                crate::draw_common::cell(ch, chunk.style),
            );
            if char_width == 2 {
                buffer.set(
                    cursor_x.saturating_add(1),
                    rect.y.saturating_add(row),
                    crate::draw_common::cell(' ', chunk.style),
                );
            }
            cursor_x = cursor_x.saturating_add(char_width);
        }
    }
}

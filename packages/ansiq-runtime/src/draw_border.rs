use ansiq_core::{
    Alignment, BlockFrame, Borders, Rect, Style, TitlePosition, patch_style, title_group_positions,
};
use ansiq_render::FrameBuffer;
use unicode_width::UnicodeWidthChar;

use crate::draw_common::cell;

pub(crate) fn draw_border(
    buffer: &mut FrameBuffer,
    rect: Rect,
    titles: &[ansiq_core::BlockTitle],
    default_title_alignment: Alignment,
    default_title_position: TitlePosition,
    borders: Borders,
    border_type: ansiq_core::BorderType,
    border_set: Option<ansiq_core::symbols::border::Set>,
    border_style: Style,
    title_style: Style,
) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }

    let symbols = border_symbols(border_type, border_set);
    let right = rect.right().saturating_sub(1);
    let bottom = rect.bottom().saturating_sub(1);

    if borders.contains(Borders::TOP) {
        for x in rect.x..=right {
            buffer.set(x, rect.y, cell(symbols.top, border_style));
        }
    }

    if borders.contains(Borders::BOTTOM) {
        for x in rect.x..=right {
            buffer.set(x, bottom, cell(symbols.bottom, border_style));
        }
    }

    if borders.contains(Borders::LEFT) {
        for y in rect.y..=bottom {
            buffer.set(rect.x, y, cell(symbols.left, border_style));
        }
    }

    if borders.contains(Borders::RIGHT) {
        for y in rect.y..=bottom {
            buffer.set(right, y, cell(symbols.right, border_style));
        }
    }

    if borders.contains(Borders::TOP | Borders::LEFT) {
        buffer.set(rect.x, rect.y, cell(symbols.top_left, border_style));
    }
    if borders.contains(Borders::TOP | Borders::RIGHT) {
        buffer.set(right, rect.y, cell(symbols.top_right, border_style));
    }
    if borders.contains(Borders::BOTTOM | Borders::LEFT) {
        buffer.set(rect.x, bottom, cell(symbols.bottom_left, border_style));
    }
    if borders.contains(Borders::BOTTOM | Borders::RIGHT) {
        buffer.set(right, bottom, cell(symbols.bottom_right, border_style));
    }

    render_titles(
        buffer,
        rect,
        titles,
        default_title_alignment,
        default_title_position,
        borders,
        title_style,
    );
}

pub(crate) fn block_inner_rect(rect: Rect, block: &BlockFrame) -> Rect {
    block.inner(rect)
}

pub(crate) fn draw_block_frame(
    buffer: &mut FrameBuffer,
    rect: Rect,
    block: &BlockFrame,
    block_style: Style,
) {
    crate::draw_common::fill_surface(buffer, rect, block_style);
    if !block.props.borders.is_empty() || !block.props.titles.is_empty() {
        draw_border(
            buffer,
            rect,
            &block.props.titles,
            block.props.title_alignment,
            block.props.title_position,
            block.props.borders,
            block.props.border_type,
            block.props.border_set,
            crate::draw_common::border_style(block_style, block.props.border_style, false),
            crate::draw_common::title_style(block_style, block.props.title_style),
        );
    }
}

fn render_titles(
    buffer: &mut FrameBuffer,
    rect: Rect,
    titles: &[ansiq_core::BlockTitle],
    default_alignment: Alignment,
    default_position: TitlePosition,
    borders: Borders,
    title_style: Style,
) {
    render_titles_for_position(
        buffer,
        rect,
        titles,
        TitlePosition::Top,
        default_alignment,
        default_position,
        borders,
        title_style,
    );
    render_titles_for_position(
        buffer,
        rect,
        titles,
        TitlePosition::Bottom,
        default_alignment,
        default_position,
        borders,
        title_style,
    );
}

fn render_titles_for_position(
    buffer: &mut FrameBuffer,
    rect: Rect,
    titles: &[ansiq_core::BlockTitle],
    position: TitlePosition,
    default_alignment: Alignment,
    default_position: TitlePosition,
    borders: Borders,
    title_style: Style,
) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }

    let relevant = titles
        .iter()
        .filter(|title| title.position.unwrap_or(default_position) == position)
        .collect::<Vec<_>>();
    if relevant.is_empty() {
        return;
    }

    let row_y = match position {
        TitlePosition::Top => rect.y,
        TitlePosition::Bottom => rect.bottom().saturating_sub(1),
    };
    let start_x = rect
        .x
        .saturating_add(u16::from(borders.contains(Borders::LEFT)));
    let end_x = rect
        .right()
        .saturating_sub(u16::from(borders.contains(Borders::RIGHT)));
    if end_x <= start_x {
        return;
    }
    let area = Rect::new(start_x, row_y, end_x.saturating_sub(start_x), 1);

    let mut left = Vec::new();
    let mut center = Vec::new();
    let mut right = Vec::new();
    for title in relevant {
        match title.content.alignment.unwrap_or(default_alignment) {
            Alignment::Left => left.push(title),
            Alignment::Center => center.push(title),
            Alignment::Right => right.push(title),
        }
    }

    let left_line = (!left.is_empty()).then(|| join_block_titles(&left));
    let center_line = (!center.is_empty()).then(|| join_block_titles(&center));
    let right_line = (!right.is_empty()).then(|| join_block_titles(&right));

    let left_width = left_line
        .as_ref()
        .map(title_line_width)
        .unwrap_or(0)
        .min(area.width);
    let center_width = center_line
        .as_ref()
        .map(title_line_width)
        .unwrap_or(0)
        .min(area.width);
    let right_width = right_line
        .as_ref()
        .map(title_line_width)
        .unwrap_or(0)
        .min(area.width);
    let positions = title_group_positions(area.width, left_width, center_width, right_width);

    if let (Some(left_line), Some(left_x)) = (left_line, positions.left_x) {
        let fake_title = ansiq_core::BlockTitle::new(left_line);
        let _ = draw_title_line(
            buffer,
            area,
            area.x.saturating_add(left_x),
            &fake_title,
            title_style,
        );
    }

    if let (Some(right_line), Some(right_x)) = (right_line, positions.right_x) {
        let fake_title = ansiq_core::BlockTitle::new(right_line);
        let _ = draw_title_line(
            buffer,
            area,
            area.x.saturating_add(right_x),
            &fake_title,
            title_style,
        );
    }

    if let (Some(center_line), Some(center_x)) = (center_line, positions.center_x) {
        let fake_title = ansiq_core::BlockTitle::new(center_line);
        let _ = draw_title_line(
            buffer,
            area,
            area.x.saturating_add(center_x),
            &fake_title,
            title_style,
        );
    }
}

fn join_block_titles(titles: &[&ansiq_core::BlockTitle]) -> ansiq_core::Line {
    let mut spans = Vec::new();
    for (index, title) in titles.iter().enumerate() {
        if index > 0 {
            spans.push(ansiq_core::Span::styled(" ", Style::default()));
        }
        spans.extend(title.content.spans.clone());
    }
    ansiq_core::Line {
        spans,
        alignment: None,
    }
}

fn title_line_width(line: &ansiq_core::Line) -> u16 {
    line.width().min(u16::MAX as usize) as u16
}

fn draw_title_line(
    buffer: &mut FrameBuffer,
    area: Rect,
    start_x: u16,
    title: &ansiq_core::BlockTitle,
    title_style: Style,
) -> u16 {
    let mut cursor_x = start_x;
    for span in &title.content.spans {
        let style = patch_style(title_style, span.style);
        for ch in span.content.chars() {
            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
            let char_width = char_width.max(1);
            if cursor_x.saturating_add(char_width) > area.right() {
                return cursor_x;
            }
            buffer.set(cursor_x, area.y, cell(ch, style));
            if char_width == 2 {
                buffer.set(cursor_x.saturating_add(1), area.y, cell(' ', style));
            }
            cursor_x = cursor_x.saturating_add(char_width);
        }
    }
    cursor_x
}

struct BorderSymbols {
    top: char,
    right: char,
    bottom: char,
    left: char,
    top_left: char,
    top_right: char,
    bottom_left: char,
    bottom_right: char,
}

fn border_symbols(
    border_type: ansiq_core::BorderType,
    border_set: Option<ansiq_core::symbols::border::Set>,
) -> BorderSymbols {
    let set = border_set.unwrap_or(match border_type {
        ansiq_core::BorderType::Plain => ansiq_core::symbols::border::PLAIN,
        ansiq_core::BorderType::Rounded => ansiq_core::symbols::border::ROUNDED,
        ansiq_core::BorderType::Double => ansiq_core::symbols::border::DOUBLE,
        ansiq_core::BorderType::Thick => ansiq_core::symbols::border::THICK,
    });

    BorderSymbols {
        top: set.horizontal_top,
        right: set.vertical_right,
        bottom: set.horizontal_bottom,
        left: set.vertical_left,
        top_left: set.top_left,
        top_right: set.top_right,
        bottom_left: set.bottom_left,
        bottom_right: set.bottom_right,
    }
}

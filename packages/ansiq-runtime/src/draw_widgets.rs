use ansiq_core::{
    Alignment, Rect, Style, StyledLine, Text, clip_to_width, patch_style, styled_line_from_line,
    styled_line_from_span, styled_lines_from_text,
};
use ansiq_render::FrameBuffer;

use crate::draw_border::{block_inner_rect, draw_block_frame};
use crate::draw_common::{cell, merge_highlight_style};
use crate::draw_text::{append_styled_line, draw_console_line, draw_styled_line};

pub(crate) fn draw_list<Message>(
    buffer: &mut FrameBuffer,
    rect: Rect,
    props: &ansiq_core::ListProps<Message>,
    style: Style,
) {
    let (inner_rect, list_style) = if let Some(block) = &props.block {
        let block_style = patch_style(style, block.style);
        draw_block_frame(buffer, rect, block, block_style);
        (block_inner_rect(rect, block), block_style)
    } else {
        (rect, style)
    };

    if inner_rect.is_empty() || props.items.is_empty() {
        return;
    }

    let visible = inner_rect.height as usize;
    let (start, end) = list_visible_bounds(props, visible);
    let gutter = props
        .highlight_symbol
        .as_ref()
        .map(|symbol| symbol.width() as u16)
        .unwrap_or(0);
    let selection_spacing = props
        .highlight_spacing
        .should_add(props.state.selected().is_some())
        && gutter > 0;
    let item_rect = if selection_spacing {
        Rect::new(
            inner_rect.x.saturating_add(gutter),
            inner_rect.y,
            inner_rect.width.saturating_sub(gutter),
            inner_rect.height,
        )
    } else {
        inner_rect
    };

    let mut cursor_y = 0u16;
    for (index, item) in props
        .items
        .iter()
        .enumerate()
        .skip(start)
        .take(end.saturating_sub(start))
    {
        let selected = props.state.selected() == Some(index);
        let item_style = if selected {
            merge_highlight_style(patch_style(list_style, item.style), props.highlight_style)
        } else {
            patch_style(list_style, item.style)
        };
        let item_lines = styled_lines_from_text(&item.content, item_style, Alignment::Left);
        let highlight_line = props
            .highlight_symbol
            .as_ref()
            .map(|symbol| styled_line_from_line(symbol, item_style));
        let item_height = item_lines.len() as u16;
        if cursor_y >= inner_rect.height {
            break;
        }
        if selection_spacing {
            let gutter_height = item_height.min(inner_rect.height.saturating_sub(cursor_y));
            for line_index in 0..gutter_height {
                let symbol = if selected && (line_index == 0 || props.repeat_highlight_symbol) {
                    highlight_line.as_ref()
                } else {
                    None
                };
                if let Some(symbol) = symbol {
                    let gutter_rect = Rect::new(
                        inner_rect.x,
                        inner_rect
                            .y
                            .saturating_add(cursor_y)
                            .saturating_add(line_index),
                        gutter,
                        1,
                    );
                    draw_styled_line(buffer, gutter_rect, 0, symbol, 0);
                }
            }
        }

        let rows: Box<dyn Iterator<Item = (u16, &StyledLine)>> = match props.direction {
            ansiq_core::ListDirection::TopToBottom => Box::new(
                item_lines
                    .iter()
                    .enumerate()
                    .map(|(line_index, line)| (cursor_y.saturating_add(line_index as u16), line)),
            ),
            ansiq_core::ListDirection::BottomToTop => {
                Box::new(item_lines.iter().enumerate().map(|(line_index, line)| {
                    (
                        inner_rect
                            .height
                            .saturating_sub(item_height)
                            .saturating_sub(cursor_y)
                            .saturating_add(line_index as u16),
                        line,
                    )
                }))
            }
        };

        for (draw_row, line) in rows {
            if draw_row >= item_rect.height {
                break;
            }
            draw_styled_line(buffer, item_rect, draw_row, line, 0);
        }

        cursor_y = cursor_y.saturating_add(item_height);
    }
}

fn list_visible_bounds<Message>(
    props: &ansiq_core::ListProps<Message>,
    max_height: usize,
) -> (usize, usize) {
    let len = props.items.len();
    if len == 0 || max_height == 0 {
        return (0, 0);
    }

    let offset = props.state.offset().min(len.saturating_sub(1));
    let mut first_visible = offset;
    let mut last_visible = offset;
    let mut height_from_offset = 0usize;

    for item in props.items.iter().skip(offset) {
        if height_from_offset.saturating_add(item.height()) > max_height {
            break;
        }

        height_from_offset = height_from_offset.saturating_add(item.height());
        last_visible += 1;
    }

    let index_to_display =
        apply_scroll_padding_to_selected_index(props, max_height, first_visible, last_visible)
            .unwrap_or(offset);

    while index_to_display >= last_visible && last_visible < len {
        height_from_offset = height_from_offset.saturating_add(props.items[last_visible].height());
        last_visible += 1;

        while height_from_offset > max_height && first_visible < last_visible {
            height_from_offset =
                height_from_offset.saturating_sub(props.items[first_visible].height());
            first_visible += 1;
        }
    }

    while index_to_display < first_visible {
        first_visible -= 1;
        height_from_offset = height_from_offset.saturating_add(props.items[first_visible].height());

        while height_from_offset > max_height && last_visible > first_visible {
            last_visible -= 1;
            height_from_offset =
                height_from_offset.saturating_sub(props.items[last_visible].height());
        }
    }

    (first_visible, last_visible)
}

fn apply_scroll_padding_to_selected_index<Message>(
    props: &ansiq_core::ListProps<Message>,
    max_height: usize,
    first_visible_index: usize,
    last_visible_index: usize,
) -> Option<usize> {
    let last_valid_index = props.items.len().saturating_sub(1);
    let selected = props.state.selected()?.min(last_valid_index);

    let mut scroll_padding = props.scroll_padding;
    while scroll_padding > 0 {
        let mut height_around_selected = 0usize;
        for index in selected.saturating_sub(scroll_padding)
            ..=selected
                .saturating_add(scroll_padding)
                .min(last_valid_index)
        {
            height_around_selected =
                height_around_selected.saturating_add(props.items[index].height());
        }

        if height_around_selected <= max_height {
            break;
        }

        scroll_padding -= 1;
    }

    Some(
        if selected
            .saturating_add(scroll_padding)
            .min(last_valid_index)
            >= last_visible_index
        {
            selected.saturating_add(scroll_padding)
        } else if selected.saturating_sub(scroll_padding) < first_visible_index {
            selected.saturating_sub(scroll_padding)
        } else {
            selected
        }
        .min(last_valid_index),
    )
}

pub(crate) fn draw_tabs<Message>(
    buffer: &mut FrameBuffer,
    rect: Rect,
    props: &ansiq_core::TabsProps<Message>,
    style: Style,
) {
    let (inner_rect, tabs_style) = if let Some(block) = &props.block {
        let block_style = patch_style(style, block.style);
        draw_block_frame(buffer, rect, block, block_style);
        (block_inner_rect(rect, block), block_style)
    } else {
        (rect, style)
    };

    if inner_rect.is_empty() {
        return;
    }

    let mut cursor_x = inner_rect.x;

    for (index, title) in props.titles.iter().enumerate() {
        if cursor_x >= inner_rect.right() {
            break;
        }

        let mut segment = StyledLine {
            chunks: Vec::new(),
            alignment: Alignment::Left,
            width: 0,
        };
        append_styled_line(
            &mut segment,
            &styled_line_from_line(&props.padding_left, tabs_style),
        );
        let title_lines =
            styled_lines_from_text(&Text::from(title.clone()), tabs_style, Alignment::Left);
        if let Some(title_line) = title_lines.first() {
            append_styled_line(&mut segment, title_line);
        }
        append_styled_line(
            &mut segment,
            &styled_line_from_line(&props.padding_right, tabs_style),
        );

        let segment_style = if Some(index) == props.selected {
            merge_highlight_style(tabs_style, props.highlight_style)
        } else {
            tabs_style
        };
        for chunk in &mut segment.chunks {
            chunk.style = patch_style(segment_style, chunk.style);
        }
        let segment_rect = Rect::new(
            cursor_x,
            inner_rect.y,
            inner_rect.right().saturating_sub(cursor_x),
            inner_rect.height.min(1),
        );
        draw_styled_line(buffer, segment_rect, 0, &segment, 0);
        cursor_x = cursor_x.saturating_add(segment.width);

        if index + 1 < props.titles.len()
            && cursor_x < inner_rect.right()
            && !props.divider.content.is_empty()
        {
            let divider = styled_line_from_span(&props.divider, tabs_style);
            let divider_rect = Rect::new(
                cursor_x,
                inner_rect.y,
                inner_rect.right().saturating_sub(cursor_x),
                inner_rect.height.min(1),
            );
            draw_styled_line(buffer, divider_rect, 0, &divider, 0);
            cursor_x = cursor_x.saturating_add(divider.width);
        }
    }
}

pub(crate) fn draw_gauge(
    buffer: &mut FrameBuffer,
    rect: Rect,
    props: &ansiq_core::GaugeProps,
    style: Style,
) {
    if rect.is_empty() {
        return;
    }

    let (gauge_rect, gauge_style) = if let Some(block) = &props.block {
        let block_style = patch_style(style, block.style);
        draw_block_frame(buffer, rect, block, block_style);
        (block_inner_rect(rect, block), block_style)
    } else {
        (rect, style)
    };

    if gauge_rect.is_empty() {
        return;
    }

    let ratio = props.ratio.clamp(0.0, 1.0);
    let exact_fill = ratio * f64::from(gauge_rect.width);
    let full = exact_fill.floor() as u16;
    let fraction = ((exact_fill - f64::from(full)) * 8.0).round() as u8;

    for y in gauge_rect.y..gauge_rect.bottom() {
        for offset in 0..gauge_rect.width {
            let x = gauge_rect.x.saturating_add(offset);
            let symbol = if offset < full {
                '█'
            } else if props.use_unicode && offset == full {
                partial_block(fraction).unwrap_or('░')
            } else {
                '░'
            };
            let cell_style =
                if offset < full || (props.use_unicode && offset == full && fraction > 0) {
                    props.gauge_style
                } else {
                    gauge_style
                };
            buffer.set(x, y, cell(symbol, cell_style));
        }
    }

    let label = props
        .label
        .clone()
        .map(ansiq_core::Line::from)
        .unwrap_or_else(|| ansiq_core::Line::from(format!("{}%", (ratio * 100.0).round() as u16)));
    let label_width = label.width() as u16;
    if label_width > 0 && label_width <= gauge_rect.width {
        let label_x = gauge_rect
            .x
            .saturating_add((gauge_rect.width.saturating_sub(label_width)) / 2);
        let label_y = gauge_rect
            .y
            .saturating_add(gauge_rect.height.saturating_sub(1) / 2);
        let label_rect = Rect::new(
            label_x,
            label_y,
            gauge_rect.width.saturating_sub(label_x - gauge_rect.x),
            1,
        );
        let label_line = styled_line_from_line(&label, gauge_style.bold(true));
        draw_styled_line(buffer, label_rect, 0, &label_line, 0);
    }
}

pub(crate) fn draw_line_gauge(
    buffer: &mut FrameBuffer,
    rect: Rect,
    props: &ansiq_core::LineGaugeProps,
    style: Style,
) {
    if rect.is_empty() {
        return;
    }

    let (line_rect, text_style) = if let Some(block) = &props.block {
        let block_style = patch_style(style, block.style);
        draw_block_frame(buffer, rect, block, block_style);
        (block_inner_rect(rect, block), block_style)
    } else {
        (rect, style)
    };

    if line_rect.is_empty() {
        return;
    }

    let label = props.label.clone().unwrap_or_else(|| {
        ansiq_core::Line::from(format!(
            "{}%",
            (props.ratio.clamp(0.0, 1.0) * 100.0).round() as u16
        ))
    });
    let label_width = label.width() as u16;
    if label_width > 0 {
        let label_line = styled_line_from_line(&label, text_style.bold(true));
        draw_styled_line(buffer, line_rect, 0, &label_line, 0);
    }

    let gap = u16::from(line_rect.width > label_width);
    let line_x = line_rect.x.saturating_add(label_width.saturating_add(gap));
    let line_width = line_rect.right().saturating_sub(line_x);
    let filled = (f64::from(line_width) * props.ratio.clamp(0.0, 1.0)).round() as u16;

    for offset in 0..line_width {
        let x = line_x.saturating_add(offset);
        let (symbol, cell_style) = if offset < filled {
            (
                props.filled_symbol.chars().next().unwrap_or('─'),
                props.filled_style,
            )
        } else {
            (
                props.unfilled_symbol.chars().next().unwrap_or('─'),
                props.unfilled_style,
            )
        };
        buffer.set(x, line_rect.y, cell(symbol, cell_style));
    }
}

pub(crate) fn draw_sparkline(
    buffer: &mut FrameBuffer,
    rect: Rect,
    props: &ansiq_core::SparklineProps,
    style: Style,
) {
    if rect.is_empty() || props.values.is_empty() {
        return;
    }

    let width = rect.width as usize;
    let start = props.values.len().saturating_sub(width);
    let visible = &props.values[start..];
    let max = props
        .max
        .unwrap_or_else(|| visible.iter().flatten().copied().max().unwrap_or(0))
        .max(1);

    for (index, value) in visible.iter().enumerate() {
        let x = match props.direction {
            ansiq_core::SparklineDirection::LeftToRight => rect.x.saturating_add(index as u16),
            ansiq_core::SparklineDirection::RightToLeft => {
                rect.right().saturating_sub(1 + index as u16)
            }
        };
        let (symbol, cell_style) = match value {
            Some(value) => {
                let level = (value.saturating_mul(8) / max).min(8) as usize;
                let symbol = match level {
                    0 => ' ',
                    1 => '▁',
                    2 => '▂',
                    3 => '▃',
                    4 => '▄',
                    5 => '▅',
                    6 => '▆',
                    7 => '▇',
                    _ => '█',
                };
                (symbol, style)
            }
            None => (props.absent_value_symbol, props.absent_value_style),
        };
        buffer.set(x, rect.y, cell(symbol, cell_style));
    }
}

pub(crate) fn draw_bar_chart(
    buffer: &mut FrameBuffer,
    rect: Rect,
    props: &ansiq_core::BarChartProps,
    style: Style,
) {
    if rect.is_empty() || props.bars.is_empty() {
        return;
    }

    let chart_height = rect.height.saturating_sub(1);
    if chart_height == 0 {
        return;
    }
    let max = props
        .max
        .unwrap_or_else(|| props.bars.iter().map(|bar| bar.value).max().unwrap_or(1))
        .max(1);
    let step = props.bar_width.max(1).saturating_add(1);

    for (index, bar) in props.bars.iter().enumerate() {
        let x = rect.x.saturating_add(index as u16 * step);
        if x >= rect.right() {
            break;
        }
        let width = props.bar_width.min(rect.right().saturating_sub(x));
        let filled = ((u32::from(chart_height) * bar.value.min(max) as u32) / max as u32) as u16;

        for dx in 0..width {
            for dy in 0..filled.min(chart_height) {
                let y = rect
                    .y
                    .saturating_add(chart_height.saturating_sub(1).saturating_sub(dy));
                buffer.set(x.saturating_add(dx), y, cell('█', style));
            }
        }

        let label = clip_to_width(&bar.label, width);
        let label_rect = Rect::new(x, rect.bottom().saturating_sub(1), width, 1);
        buffer.write_clipped(label_rect, 0, 0, &label, style);
    }
}

pub(crate) fn draw_chart(
    buffer: &mut FrameBuffer,
    rect: Rect,
    props: &ansiq_core::ChartProps,
    style: Style,
) {
    if rect.width < 2 || rect.height < 2 {
        return;
    }

    let plot_left = rect.x;
    let plot_bottom = rect.bottom().saturating_sub(1);
    for y in rect.y..rect.bottom() {
        buffer.set(plot_left, y, cell('│', style));
    }
    for x in rect.x..rect.right() {
        buffer.set(x, plot_bottom, cell('─', style));
    }
    buffer.set(plot_left, plot_bottom, cell('└', style));

    let points: Vec<(i64, i64)> = props
        .datasets
        .iter()
        .flat_map(|dataset| dataset.points.iter().copied())
        .collect();
    if points.is_empty() {
        return;
    }

    let min_x = points.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let max_x = points
        .iter()
        .map(|(x, _)| *x)
        .max()
        .unwrap_or(min_x)
        .max(min_x + 1);
    let min_y = props
        .min_y
        .unwrap_or_else(|| points.iter().map(|(_, y)| *y).min().unwrap_or(0));
    let max_y = props
        .max_y
        .unwrap_or_else(|| points.iter().map(|(_, y)| *y).max().unwrap_or(min_y))
        .max(min_y + 1);
    let plot_width = rect.width.saturating_sub(1).max(1);
    let plot_height = rect.height.saturating_sub(1).max(1);

    for dataset in &props.datasets {
        for (x, y) in &dataset.points {
            let px = scale_i64(*x, min_x, max_x, plot_width.saturating_sub(1));
            let py = scale_i64(*y, min_y, max_y, plot_height.saturating_sub(1));
            let draw_x = rect.x.saturating_add(1).saturating_add(px);
            let draw_y = plot_bottom.saturating_sub(py);
            if draw_x < rect.right() && draw_y < rect.bottom() {
                buffer.set(draw_x, draw_y, cell('•', style.bold(true)));
            }
        }
    }
}

pub(crate) fn draw_canvas(
    buffer: &mut FrameBuffer,
    rect: Rect,
    props: &ansiq_core::CanvasProps,
    fallback_style: Style,
) {
    if rect.is_empty() || props.width == 0 || props.height == 0 {
        return;
    }

    for cell_data in &props.cells {
        let x = rect.x.saturating_add(cell_data.x);
        let y = rect.y.saturating_add(cell_data.y);
        if x < rect.right() && y < rect.bottom() {
            let style = if cell_data.style == Style::default() {
                fallback_style
            } else {
                cell_data.style
            };
            buffer.set(x, y, cell(cell_data.symbol, style));
        }
    }
}

pub(crate) fn draw_monthly(
    buffer: &mut FrameBuffer,
    rect: Rect,
    props: &ansiq_core::MonthlyProps,
    style: Style,
) {
    if rect.height < 2 || rect.width == 0 {
        return;
    }

    buffer.write_clipped(
        rect,
        0,
        0,
        &format!("{:04}-{:02}", props.year, props.month),
        style.bold(true),
    );
    if rect.height < 3 {
        return;
    }
    buffer.write_clipped(rect, 0, 1, "Mo Tu We Th Fr Sa Su", style);

    let first_weekday = first_weekday_monday(props.year, props.month);
    let days = days_in_month(props.year, props.month);
    let mut day = 1u8;
    for week in 0..6u16 {
        let y = rect.y.saturating_add(2 + week);
        if y >= rect.bottom() || day > days {
            break;
        }
        let mut line = String::new();
        for weekday in 0..7u8 {
            if week == 0 && weekday < first_weekday {
                line.push_str("   ");
                continue;
            }
            if day > days {
                line.push_str("   ");
                continue;
            }
            line.push_str(&format!("{day:>2} "));
            day = day.saturating_add(1);
        }
        draw_console_line(buffer, rect, 2 + week, line.trim_end(), style);
    }
}

fn partial_block(fraction: u8) -> Option<char> {
    match fraction {
        1 => Some('▏'),
        2 => Some('▎'),
        3 => Some('▍'),
        4 => Some('▌'),
        5 => Some('▋'),
        6 => Some('▊'),
        7 => Some('▉'),
        _ => None,
    }
}

fn scale_i64(value: i64, min: i64, max: i64, extent: u16) -> u16 {
    if max <= min || extent == 0 {
        return 0;
    }
    let clamped = value.clamp(min, max) - min;
    let range = (max - min) as u64;
    ((clamped as u64 * u64::from(extent)) / range) as u16
}

fn first_weekday_monday(year: i32, month: u8) -> u8 {
    let mut m = i32::from(month);
    let mut y = year;
    if m < 3 {
        m += 12;
        y -= 1;
    }
    let q = 1i32;
    let k = y % 100;
    let j = y / 100;
    let h = (q + ((13 * (m + 1)) / 5) + k + (k / 4) + (j / 4) + (5 * j)) % 7;
    match h {
        0 => 5,
        1 => 6,
        n => (n - 2) as u8,
    }
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 30,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

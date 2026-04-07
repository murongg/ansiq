use ansiq_core::{Rect, ScrollbarOrientation, ScrollbarProps, Style, patch_style};
use ansiq_render::FrameBuffer;

use crate::draw_common::cell;

pub(crate) fn draw_scrollbar<Message>(
    buffer: &mut FrameBuffer,
    rect: Rect,
    props: &ScrollbarProps<Message>,
    style: Style,
) {
    if rect.is_empty() {
        return;
    }

    let content_length = props.state.content_length_value();
    let axis_length = match props.orientation {
        ScrollbarOrientation::VerticalRight | ScrollbarOrientation::VerticalLeft => {
            rect.height.max(1)
        }
        ScrollbarOrientation::HorizontalBottom | ScrollbarOrientation::HorizontalTop => {
            rect.width.max(1)
        }
    };
    let begin_len = u16::from(props.begin_symbol.is_some());
    let end_len = u16::from(props.end_symbol.is_some());
    let track_length = axis_length
        .saturating_sub(begin_len)
        .saturating_sub(end_len);
    if content_length == 0 || track_length == 0 {
        return;
    }

    let viewport_length = if props.state.viewport_content_length_value() != 0 {
        props.state.viewport_content_length_value()
    } else {
        match props.orientation {
            ScrollbarOrientation::VerticalRight | ScrollbarOrientation::VerticalLeft => {
                rect.height as usize
            }
            ScrollbarOrientation::HorizontalBottom | ScrollbarOrientation::HorizontalTop => {
                rect.width as usize
            }
        }
    };

    let max_position = content_length.saturating_sub(1);
    let start_position = props.state.get_position().min(max_position);
    let max_viewport_position = max_position.saturating_add(viewport_length);
    if max_viewport_position == 0 {
        return;
    }

    let thumb_length = rounding_divide(
        viewport_length.saturating_mul(track_length as usize),
        max_viewport_position,
    )
    .clamp(1, track_length as usize) as u16;
    let thumb_start = rounding_divide(
        start_position.saturating_mul(track_length as usize),
        max_viewport_position,
    )
    .clamp(0, track_length.saturating_sub(1) as usize) as u16;
    let track_end = track_length.saturating_sub(thumb_start.saturating_add(thumb_length));

    let thumb_style = patch_style(style, props.thumb_style);
    let track_style = patch_style(style, props.track_style);
    let begin_style = patch_style(style, props.begin_style);
    let end_style = patch_style(style, props.end_style);

    if let Some(symbol) = props
        .begin_symbol
        .as_ref()
        .and_then(|symbol| symbol.chars().next())
    {
        match props.orientation {
            ScrollbarOrientation::VerticalRight | ScrollbarOrientation::VerticalLeft => {
                buffer.set(rect.x, rect.y, cell(symbol, begin_style));
            }
            ScrollbarOrientation::HorizontalBottom | ScrollbarOrientation::HorizontalTop => {
                buffer.set(rect.x, rect.y, cell(symbol, begin_style));
            }
        }
    }

    if let Some(symbol) = props
        .end_symbol
        .as_ref()
        .and_then(|symbol| symbol.chars().next())
    {
        match props.orientation {
            ScrollbarOrientation::VerticalRight | ScrollbarOrientation::VerticalLeft => {
                buffer.set(
                    rect.x,
                    rect.bottom().saturating_sub(1),
                    cell(symbol, end_style),
                );
            }
            ScrollbarOrientation::HorizontalBottom | ScrollbarOrientation::HorizontalTop => {
                buffer.set(
                    rect.right().saturating_sub(1),
                    rect.y,
                    cell(symbol, end_style),
                );
            }
        }
    }

    let track_symbol = props
        .track_symbol
        .as_ref()
        .and_then(|symbol| symbol.chars().next());
    let thumb_symbol = props.thumb_symbol.chars().next().unwrap_or('█');
    let track_start = thumb_start;

    for offset in 0..track_start {
        let Some(symbol) = track_symbol else {
            break;
        };
        let axis_offset = begin_len.saturating_add(offset);
        draw_scrollbar_symbol(
            buffer,
            rect,
            props.orientation,
            axis_offset,
            symbol,
            track_style,
        );
    }

    for offset in 0..thumb_length {
        let axis_offset = begin_len.saturating_add(track_start).saturating_add(offset);
        draw_scrollbar_symbol(
            buffer,
            rect,
            props.orientation,
            axis_offset,
            thumb_symbol,
            thumb_style,
        );
    }

    for offset in 0..track_end {
        let Some(symbol) = track_symbol else {
            break;
        };
        let axis_offset = begin_len
            .saturating_add(track_start)
            .saturating_add(thumb_length)
            .saturating_add(offset);
        draw_scrollbar_symbol(
            buffer,
            rect,
            props.orientation,
            axis_offset,
            symbol,
            track_style,
        );
    }
}

fn draw_scrollbar_symbol(
    buffer: &mut FrameBuffer,
    rect: Rect,
    orientation: ScrollbarOrientation,
    axis_offset: u16,
    symbol: char,
    symbol_style: Style,
) {
    match orientation {
        ScrollbarOrientation::VerticalRight | ScrollbarOrientation::VerticalLeft => {
            buffer.set(
                rect.x,
                rect.y.saturating_add(axis_offset),
                cell(symbol, symbol_style),
            );
        }
        ScrollbarOrientation::HorizontalBottom | ScrollbarOrientation::HorizontalTop => {
            buffer.set(
                rect.x.saturating_add(axis_offset),
                rect.y,
                cell(symbol, symbol_style),
            );
        }
    }
}

const fn rounding_divide(numerator: usize, denominator: usize) -> usize {
    (numerator + denominator / 2) / denominator
}

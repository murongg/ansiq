use ansiq_core::{
    Alignment, Rect, Style, TableAlignment, TableProps, patch_style, styled_lines_from_text,
    table_column_layout, table_span_width,
};
use ansiq_render::FrameBuffer;

use crate::draw_border::{block_inner_rect, draw_block_frame};
use crate::draw_common::merge_highlight_style;
use crate::draw_text::draw_styled_line;

pub(crate) fn draw_table<Message>(
    buffer: &mut FrameBuffer,
    rect: Rect,
    props: &TableProps<Message>,
    style: Style,
) {
    if rect.is_empty() {
        return;
    }

    let (table_rect, table_style) = if let Some(block) = &props.block {
        let block_style = patch_style(style, block.style);
        draw_block_frame(buffer, rect, block, block_style);
        (block_inner_rect(rect, block), block_style)
    } else {
        (rect, style)
    };

    let column_count = props
        .header
        .as_ref()
        .map(ansiq_core::Row::column_count)
        .unwrap_or(0)
        .max(
            props
                .footer
                .as_ref()
                .map(ansiq_core::Row::column_count)
                .unwrap_or(0),
        )
        .max(
            props
                .rows
                .iter()
                .map(ansiq_core::Row::column_count)
                .max()
                .unwrap_or(0),
        );
    if column_count == 0 {
        return;
    }

    let highlight_symbol = props
        .highlight_symbol
        .as_ref()
        .and_then(|text| text.lines.first());
    let gutter = highlight_symbol
        .map(|line| line.width() as u16)
        .unwrap_or(0);
    let selection_spacing = props
        .highlight_spacing
        .should_add(props.state.selected().is_some())
        && gutter > 0;
    let table_rect = if selection_spacing {
        Rect::new(
            rect.x.saturating_add(gutter),
            rect.y,
            rect.width.saturating_sub(gutter),
            rect.height,
        )
    } else {
        table_rect
    };
    let (column_widths, column_positions) = table_column_layout(
        table_rect.width,
        column_count,
        &props.widths,
        props.column_spacing,
        props.flex,
    );
    let used_width = column_widths
        .iter()
        .zip(column_positions.iter())
        .map(|(width, position)| position.saturating_add(*width))
        .max()
        .unwrap_or(0)
        .min(table_rect.width);
    let table_rect = Rect::new(table_rect.x, table_rect.y, used_width, table_rect.height);
    let mut line_index = 0u16;

    if let Some(header) = &props.header {
        draw_table_row(
            buffer,
            table_rect,
            line_index,
            header,
            &column_widths,
            &column_positions,
            &props.alignments,
            table_style.bold(true),
            props.column_highlight_style,
            props.cell_highlight_style,
            None,
            None,
        );
        line_index = line_index.saturating_add(header.height_with_margin());
    }

    let footer_height = props
        .footer
        .as_ref()
        .map(ansiq_core::Row::height_with_margin)
        .unwrap_or(0);
    let body_height = table_rect
        .height
        .saturating_sub(line_index)
        .saturating_sub(footer_height);
    if body_height == 0 {
        if let Some(footer) = &props.footer {
            let footer_y = table_rect
                .height
                .saturating_sub(footer.height_with_margin());
            draw_table_row(
                buffer,
                table_rect,
                footer_y,
                footer,
                &column_widths,
                &column_positions,
                &props.alignments,
                table_style.bold(true),
                props.column_highlight_style,
                props.cell_highlight_style,
                None,
                None,
            );
        }
        return;
    }

    let mut start = props.state.offset().min(props.rows.len().saturating_sub(1));
    if let Some(selected) = props
        .state
        .selected()
        .map(|index| index.min(props.rows.len().saturating_sub(1)))
    {
        if selected < start {
            start = selected;
        } else {
            while !table_row_is_visible(&props.rows, start, selected, body_height)
                && start < selected
            {
                start = start.saturating_add(1);
            }
        }
    }

    let selected_cell = props.state.selected().zip(props.state.selected_column());
    let mut draw_y = line_index;
    let body_end = line_index.saturating_add(body_height);
    for (index, row) in props.rows.iter().enumerate().skip(start) {
        let row_height = row.height_with_margin();
        if row_height == 0 || draw_y >= body_end {
            break;
        }
        if draw_y.saturating_add(row_height) > body_end {
            break;
        }

        let selected = props.state.selected() == Some(index);
        let row_style = if selected {
            merge_highlight_style(table_style, props.row_highlight_style)
        } else {
            table_style
        };

        if selection_spacing {
            let gutter_rect = Rect::new(
                table_rect.x.saturating_sub(gutter),
                table_rect
                    .y
                    .saturating_add(draw_y.saturating_add(row.top_margin_value())),
                gutter,
                1,
            );
            if selected {
                if let Some(symbol) = highlight_symbol {
                    let symbol = ansiq_core::styled_line_from_line(symbol, row_style);
                    draw_styled_line(buffer, gutter_rect, 0, &symbol, 0);
                }
            }
        }

        draw_table_row(
            buffer,
            table_rect,
            draw_y,
            row,
            &column_widths,
            &column_positions,
            &props.alignments,
            row_style,
            props.column_highlight_style,
            props.cell_highlight_style,
            props.state.selected_column(),
            selected_cell.filter(|(selected_row, _)| *selected_row == index),
        );
        draw_y = draw_y.saturating_add(row_height);
    }

    if let Some(footer) = &props.footer {
        let footer_y = table_rect
            .height
            .saturating_sub(footer.height_with_margin());
        draw_table_row(
            buffer,
            table_rect,
            footer_y,
            footer,
            &column_widths,
            &column_positions,
            &props.alignments,
            table_style.bold(true),
            props.column_highlight_style,
            props.cell_highlight_style,
            None,
            None,
        );
    }
}

fn draw_table_row(
    buffer: &mut FrameBuffer,
    table_rect: Rect,
    row_y: u16,
    row: &ansiq_core::Row,
    column_widths: &[u16],
    column_positions: &[u16],
    alignments: &[ansiq_core::TableAlignment],
    row_style: Style,
    column_highlight_style: Style,
    cell_highlight_style: Style,
    selected_column: Option<usize>,
    selected_cell: Option<(usize, usize)>,
) {
    let content_y = row_y.saturating_add(row.top_margin_value());
    let content_height = row.height_value();
    if content_height == 0 {
        return;
    }

    let mut column_index = 0usize;
    for cell in row.cells_ref() {
        let span = cell.column_span_value().max(1) as usize;
        let Some(&column_x) = column_positions.get(column_index) else {
            column_index = column_index.saturating_add(span);
            continue;
        };
        let x = table_rect.x.saturating_add(column_x);
        let cell_width = table_span_width(
            column_widths,
            column_positions,
            table_rect.width,
            column_index,
            span,
        );
        if cell_width == 0 || x >= table_rect.right() {
            column_index = column_index.saturating_add(span);
            continue;
        }

        let cell_rect = Rect::new(
            x,
            table_rect.y.saturating_add(content_y),
            cell_width.min(table_rect.right().saturating_sub(x)),
            content_height.min(
                table_rect
                    .bottom()
                    .saturating_sub(table_rect.y.saturating_add(content_y)),
            ),
        );
        let mut cell_style = patch_style(row_style, cell.style_value());
        if selected_column
            .map(|selected| (column_index..column_index.saturating_add(span)).contains(&selected))
            .unwrap_or(false)
        {
            cell_style = merge_highlight_style(cell_style, column_highlight_style);
        }
        if let Some((_, selected_column)) = selected_cell {
            if (column_index..column_index.saturating_add(span)).contains(&selected_column) {
                cell_style = merge_highlight_style(cell_style, cell_highlight_style);
            }
        }

        let alignment = cell
            .text()
            .alignment
            .unwrap_or_else(|| table_alignment(alignments.get(column_index).copied()));
        let lines = styled_lines_from_text(cell.text(), cell_style, alignment);
        for (line_index, line) in lines.into_iter().take(content_height as usize).enumerate() {
            draw_styled_line(buffer, cell_rect, line_index as u16, &line, 0);
        }

        column_index = column_index.saturating_add(span);
    }
}

fn table_alignment(alignment: Option<TableAlignment>) -> Alignment {
    match alignment.unwrap_or(TableAlignment::Left) {
        TableAlignment::Left => Alignment::Left,
        TableAlignment::Center => Alignment::Center,
        TableAlignment::Right => Alignment::Right,
    }
}

fn table_row_is_visible(
    rows: &[ansiq_core::Row],
    start: usize,
    target: usize,
    available_height: u16,
) -> bool {
    if target < start {
        return false;
    }

    let mut used = 0u16;
    for (index, row) in rows.iter().enumerate().skip(start) {
        let row_height = row.height_with_margin().max(1);
        if used.saturating_add(row_height) > available_height {
            return false;
        }
        if index == target {
            return true;
        }
        used = used.saturating_add(row_height);
    }

    false
}

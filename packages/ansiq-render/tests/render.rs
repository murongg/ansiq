use ansiq_core::{Color, HistoryEntry, Rect, Style, history_block_from_text};
use ansiq_render::{
    Cell, FrameBuffer, diff_buffers, diff_buffers_in_regions, frame_patches,
    history_block_from_buffer, render_cursor, render_history_entries, render_patches,
    render_patches_at_origin,
};

#[test]
fn diff_only_reports_changed_runs() {
    let prev = FrameBuffer::new(4, 1);
    let mut next = FrameBuffer::new(4, 1);
    next.write_str(0, 0, "flux", Style::default());

    let patches = diff_buffers(&prev, &next);

    assert_eq!(patches.len(), 1);
    assert_eq!(patches[0].text, "flux");
}

#[test]
fn identical_buffers_produce_no_patches() {
    let prev = FrameBuffer::new(4, 1);
    let next = FrameBuffer::new(4, 1);

    assert!(diff_buffers(&prev, &next).is_empty());
}

#[test]
fn frame_patches_cover_blank_cells_in_a_fresh_frame() {
    let mut frame = FrameBuffer::new(4, 2);
    frame.write_str(1, 0, "A", Style::default());

    let patches = frame_patches(&frame);

    assert_eq!(patches.len(), 2);
    assert_eq!(patches[0].x, 0);
    assert_eq!(patches[0].y, 0);
    assert_eq!(patches[0].text, " A  ");
    assert_eq!(patches[1].text, "    ");
}

#[test]
fn diff_covers_new_area_when_next_buffer_is_larger() {
    let prev = FrameBuffer::new(2, 1);
    let mut next = FrameBuffer::new(4, 1);
    next.write_str(0, 0, "flux", Style::default());

    let patches = diff_buffers(&prev, &next);

    assert_eq!(patches.len(), 1);
    assert_eq!(patches[0].text, "flux");
}

#[test]
fn write_clipped_respects_rect_and_offset() {
    let mut buffer = FrameBuffer::new(6, 2);
    buffer.write_clipped(Rect::new(1, 0, 3, 1), 1, 0, "flux", Style::default());

    assert_eq!(buffer.get(1, 0).symbol, ' ');
    assert_eq!(buffer.get(2, 0).symbol, 'f');
    assert_eq!(buffer.get(3, 0).symbol, 'l');
    assert_eq!(buffer.get(4, 0).symbol, ' ');
}

#[test]
fn diff_splits_runs_when_style_changes() {
    let prev = FrameBuffer::new(2, 1);
    let mut next = FrameBuffer::new(2, 1);
    next.set(
        0,
        0,
        Cell {
            symbol: 'a',
            style: Style::default(),
        },
    );
    next.set(
        1,
        0,
        Cell {
            symbol: 'b',
            style: Style::default().bold(true),
        },
    );

    let patches = diff_buffers(&prev, &next);

    assert_eq!(patches.len(), 2);
    assert_eq!(patches[0].text, "a");
    assert_eq!(patches[1].text, "b");
}

#[test]
fn render_patches_preserves_wide_glyphs_without_extra_spacing() {
    let prev = FrameBuffer::new(12, 1);
    let mut next = FrameBuffer::new(12, 1);
    next.write_str(0, 0, "大河向东流", Style::default());

    let patches = diff_buffers(&prev, &next);
    let mut bytes = Vec::new();
    render_patches(&mut bytes, &patches).expect("patch rendering should succeed");
    let ansi = String::from_utf8(bytes).expect("ansi output should be utf-8");
    let plain = strip_ansi(&ansi);

    assert!(plain.contains("大河向东流"));
}

#[test]
fn render_cursor_emits_show_and_move_sequences() {
    let mut bytes = Vec::new();
    render_cursor(&mut bytes, Some((4, 2))).expect("cursor rendering should succeed");
    let ansi = String::from_utf8(bytes).expect("ansi output should be utf-8");

    assert!(ansi.contains("\u{1b}[?25h"));
    assert!(ansi.contains("\u{1b}[3;5H"));
}

#[test]
fn render_cursor_hides_when_no_position_is_available() {
    let mut bytes = Vec::new();
    render_cursor(&mut bytes, None).expect("cursor rendering should succeed");
    let ansi = String::from_utf8(bytes).expect("ansi output should be utf-8");

    assert!(ansi.contains("\u{1b}[?25l"));
}

#[test]
fn render_patches_can_offset_output_below_existing_terminal_content() {
    let prev = FrameBuffer::new(4, 1);
    let mut next = FrameBuffer::new(4, 1);
    next.write_str(0, 0, "flux", Style::default());

    let patches = diff_buffers(&prev, &next);
    let mut bytes = Vec::new();
    render_patches_at_origin(&mut bytes, &patches, 7)
        .expect("offset patch rendering should succeed");
    let ansi = String::from_utf8(bytes).expect("ansi output should be utf-8");

    assert!(ansi.contains("\u{1b}[8;1H"));
}

#[test]
fn diff_buffers_in_regions_limits_patches_to_the_requested_rects() {
    let prev = FrameBuffer::new(8, 2);
    let mut next = FrameBuffer::new(8, 2);
    next.write_str(0, 0, "left", Style::default());
    next.write_str(0, 1, "right", Style::default());

    let patches = diff_buffers_in_regions(
        &prev,
        &next,
        &[Rect::new(0, 0, 4, 1), Rect::new(0, 1, 0, 0)],
    );

    assert_eq!(patches.len(), 1);
    assert_eq!(patches[0].y, 0);
    assert_eq!(patches[0].text, "left");
}

#[test]
fn history_block_from_buffer_preserves_runs_and_trims_trailing_space() {
    let mut buffer = FrameBuffer::new(6, 2);
    buffer.write_str(0, 0, "╭─╮", Style::default().fg(Color::Grey));
    buffer.write_str(0, 1, "hi", Style::default().fg(Color::White).bold(true));

    let block = history_block_from_buffer(&buffer);

    assert_eq!(block.lines.len(), 2);
    assert_eq!(block.lines[0].runs[0].text, "╭─╮");
    assert_eq!(block.lines[0].runs[0].style.fg, Color::Grey);
    assert_eq!(block.lines[1].runs[0].text, "hi");
    assert!(block.lines[1].runs[0].style.bold);
}

#[test]
fn render_history_entries_preserves_styled_block_lines() {
    let mut buffer = FrameBuffer::new(4, 1);
    buffer.write_str(0, 0, "flux", Style::default().fg(Color::Cyan));
    let block = history_block_from_buffer(&buffer);

    let mut bytes = Vec::new();
    let rendered_rows = render_history_entries(
        &mut bytes,
        &[
            HistoryEntry::Block(block),
            HistoryEntry::Text("tail".to_string()),
        ],
        80,
    )
    .expect("history rendering should succeed");
    let ansi = String::from_utf8(bytes).expect("ansi output should be utf-8");
    let plain = strip_ansi(&ansi);

    assert_eq!(rendered_rows, 3);
    assert!(plain.contains("flux"));
    assert!(plain.contains("tail"));
    assert!(ansi.contains("\u{1b}["));
}

#[test]
fn render_history_entries_returns_the_actual_cursor_row_advance() {
    let mut bytes = Vec::new();
    let rendered_rows = render_history_entries(
        &mut bytes,
        &[
            HistoryEntry::Text("wrapped".to_string()),
            HistoryEntry::Text("tail".to_string()),
        ],
        4,
    )
    .expect("history rendering should succeed");

    assert_eq!(rendered_rows, 4);
    assert_eq!(
        strip_ansi(&String::from_utf8(bytes).expect("ansi output should be utf-8")),
        "wrap\r\nped\r\n\r\ntail\r\n"
    );
}

#[test]
fn render_history_entries_renders_text_entries_like_commit_time_wrapped_blocks() {
    let mut text_bytes = Vec::new();
    let text_rows = render_history_entries(
        &mut text_bytes,
        &[HistoryEntry::Text("wrapped".to_string())],
        4,
    )
    .expect("text history rendering should succeed");

    let mut block_bytes = Vec::new();
    let block_rows = render_history_entries(
        &mut block_bytes,
        &[HistoryEntry::Block(history_block_from_text("wrapped", 4))],
        4,
    )
    .expect("block history rendering should succeed");

    assert_eq!(text_rows, block_rows);
    assert_eq!(text_bytes, block_bytes);
}

#[test]
fn render_patches_support_indexed_and_truecolor_sequences() {
    let mut bytes = Vec::new();
    render_patches(
        &mut bytes,
        &[
            ansiq_render::Patch {
                x: 0,
                y: 0,
                text: "idx".to_string(),
                style: Style::default().fg(Color::Indexed(202)),
            },
            ansiq_render::Patch {
                x: 0,
                y: 1,
                text: "rgb".to_string(),
                style: Style::default().bg(Color::Rgb(12, 34, 56)),
            },
        ],
    )
    .expect("rendering indexed and truecolor patches should succeed");
    let ansi = String::from_utf8(bytes).expect("ansi output should be utf-8");

    assert!(ansi.contains("\u{1b}[38;5;202m"));
    assert!(ansi.contains("\u{1b}[48;2;12;34;56m"));
}

fn strip_ansi(input: &str) -> String {
    let mut plain = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if chars.next_if_eq(&'[').is_some() {
                while let Some(next) = chars.next() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
            }
            continue;
        }

        plain.push(ch);
    }

    plain
}

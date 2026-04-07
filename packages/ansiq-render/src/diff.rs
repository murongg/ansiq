use std::io::{self, Write};

use ansiq_core::{
    Color, HistoryBlock, HistoryEntry, HistoryLine, HistoryRun, Style, history_block_from_text,
};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    queue,
    style::{
        Attribute, Color as CrosstermColor, Print, SetAttribute, SetBackgroundColor,
        SetForegroundColor,
    },
};
use unicode_width::UnicodeWidthChar;

use crate::{Cell, FrameBuffer};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Patch {
    pub x: u16,
    pub y: u16,
    pub text: String,
    pub style: Style,
}

pub fn diff_buffers(prev: &FrameBuffer, next: &FrameBuffer) -> Vec<Patch> {
    let width = prev.width().max(next.width());
    let height = prev.height().max(next.height());
    let mut patches = Vec::new();

    for y in 0..height {
        let mut x = 0;
        while x < width {
            let prev_cell = read_cell(prev, x, y);
            let next_cell = read_cell(next, x, y);
            if prev_cell == next_cell {
                x += 1;
                continue;
            }

            let style = next_cell.style;
            let start = x;
            let mut text = String::new();

            while x < width {
                let prev_cell = read_cell(prev, x, y);
                let next_cell = read_cell(next, x, y);
                if prev_cell == next_cell || next_cell.style != style {
                    break;
                }

                text.push(next_cell.symbol);
                x += 1;
            }

            patches.push(Patch {
                x: start,
                y,
                text,
                style,
            });
        }
    }

    patches
}

pub fn frame_patches(frame: &FrameBuffer) -> Vec<Patch> {
    let mut patches = Vec::with_capacity(frame.height() as usize);

    for y in 0..frame.height() {
        let mut x = 0;
        while x < frame.width() {
            let first = frame.get(x, y);
            let style = first.style;
            let start = x;
            let mut text = String::new();

            while x < frame.width() {
                let cell = frame.get(x, y);
                if cell.style != style {
                    break;
                }
                text.push(cell.symbol);
                x += 1;
            }

            patches.push(Patch {
                x: start,
                y,
                text,
                style,
            });
        }
    }

    patches
}

pub fn diff_buffers_in_regions(
    prev: &FrameBuffer,
    next: &FrameBuffer,
    regions: &[ansiq_core::Rect],
) -> Vec<Patch> {
    let mut patches = Vec::new();

    for region in regions {
        if region.is_empty() {
            continue;
        }

        let max_y = region.bottom().min(prev.height().max(next.height()));
        let max_x = region.right().min(prev.width().max(next.width()));

        for y in region.y..max_y {
            let mut x = region.x;
            while x < max_x {
                let prev_cell = read_cell(prev, x, y);
                let next_cell = read_cell(next, x, y);
                if prev_cell == next_cell {
                    x += 1;
                    continue;
                }

                let style = next_cell.style;
                let start = x;
                let mut text = String::new();

                while x < max_x {
                    let prev_cell = read_cell(prev, x, y);
                    let next_cell = read_cell(next, x, y);
                    if prev_cell == next_cell || next_cell.style != style {
                        break;
                    }

                    text.push(next_cell.symbol);
                    x += 1;
                }

                patches.push(Patch {
                    x: start,
                    y,
                    text,
                    style,
                });
            }
        }
    }

    patches
}

pub fn render_patches<W: Write>(writer: &mut W, patches: &[Patch]) -> io::Result<()> {
    render_patches_at_origin(writer, patches, 0)
}

pub fn render_patches_at_origin<W: Write>(
    writer: &mut W,
    patches: &[Patch],
    origin_y: u16,
) -> io::Result<()> {
    for patch in patches {
        let text = collapse_wide_continuations(&patch.text);
        queue!(
            writer,
            MoveTo(patch.x, patch.y.saturating_add(origin_y)),
            SetForegroundColor(map_color(patch.style.fg)),
            SetBackgroundColor(map_color(patch.style.bg)),
            SetAttribute(if patch.style.bold {
                Attribute::Bold
            } else {
                Attribute::NormalIntensity
            }),
            SetAttribute(if patch.style.reversed {
                Attribute::Reverse
            } else {
                Attribute::NoReverse
            }),
            Print(text)
        )?;
    }

    queue!(
        writer,
        SetForegroundColor(CrosstermColor::Reset),
        SetBackgroundColor(CrosstermColor::Reset),
        SetAttribute(Attribute::Reset)
    )?;

    writer.flush()
}

pub fn render_cursor<W: Write>(writer: &mut W, cursor: Option<(u16, u16)>) -> io::Result<()> {
    render_cursor_at_origin(writer, cursor, 0)
}

pub fn render_cursor_at_origin<W: Write>(
    writer: &mut W,
    cursor: Option<(u16, u16)>,
    origin_y: u16,
) -> io::Result<()> {
    match cursor {
        Some((x, y)) => {
            queue!(writer, Show, MoveTo(x, y.saturating_add(origin_y)))?;
        }
        None => {
            queue!(writer, Hide)?;
        }
    }

    writer.flush()
}

pub fn history_block_from_buffer(buffer: &FrameBuffer) -> HistoryBlock {
    let mut lines = Vec::with_capacity(buffer.height() as usize);

    for y in 0..buffer.height() {
        let mut last_visible_x = None;
        for x in 0..buffer.width() {
            if buffer.get(x, y).symbol != ' ' {
                last_visible_x = Some(x);
            }
        }

        let Some(end_x) = last_visible_x.map(|x| x + 1) else {
            lines.push(HistoryLine { runs: Vec::new() });
            continue;
        };

        let mut runs = Vec::new();
        let mut current_style = buffer.get(0, y).style;
        let mut current_text = String::new();

        for x in 0..end_x {
            let cell = buffer.get(x, y);
            if x == 0 {
                current_style = cell.style;
            }
            if cell.style != current_style {
                push_history_run(&mut runs, &mut current_text, current_style);
                current_style = cell.style;
            }
            current_text.push(cell.symbol);
        }

        push_history_run(&mut runs, &mut current_text, current_style);
        lines.push(HistoryLine { runs });
    }

    HistoryBlock { lines }
}

pub fn render_history_entries<W: Write>(
    writer: &mut W,
    entries: &[HistoryEntry],
    width: u16,
) -> io::Result<u16> {
    let mut wrote_any_line = false;
    let mut rendered_rows = 0u16;

    for (entry_index, entry) in entries.iter().enumerate() {
        let owned_block;
        let block = match entry {
            HistoryEntry::Text(content) => {
                owned_block = history_block_from_text(content, width);
                &owned_block
            }
            HistoryEntry::Block(block) => block,
        };

        if entry_index > 0 && wrote_any_line {
            write!(writer, "\r\n")?;
            rendered_rows = rendered_rows.saturating_add(1);
        }

        for (line_index, line) in block.lines.iter().enumerate() {
            if wrote_any_line || line_index > 0 {
                write!(writer, "\r\n")?;
            }
            render_history_line(writer, line)?;
            wrote_any_line = true;
            rendered_rows = rendered_rows.saturating_add(1);
        }
    }

    if rendered_rows > 0 {
        write!(writer, "\r\n")?;
    }
    queue!(
        writer,
        SetForegroundColor(CrosstermColor::Reset),
        SetBackgroundColor(CrosstermColor::Reset),
        SetAttribute(Attribute::Reset)
    )?;
    writer.flush()?;
    Ok(rendered_rows)
}

fn collapse_wide_continuations(text: &str) -> String {
    let mut collapsed = String::new();
    let mut skip = 0u16;

    for ch in text.chars() {
        if skip > 0 {
            skip -= 1;
            continue;
        }

        collapsed.push(ch);
        let width = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
        if width > 1 {
            skip = width - 1;
        }
    }

    collapsed
}

fn push_history_run(runs: &mut Vec<HistoryRun>, text: &mut String, style: Style) {
    if text.is_empty() {
        return;
    }

    runs.push(HistoryRun {
        text: collapse_wide_continuations(text),
        style,
    });
    text.clear();
}

fn render_history_line<W: Write>(writer: &mut W, line: &HistoryLine) -> io::Result<()> {
    if line.runs.is_empty() {
        return Ok(());
    }

    for run in &line.runs {
        queue!(
            writer,
            SetForegroundColor(map_color(run.style.fg)),
            SetBackgroundColor(map_color(run.style.bg)),
            SetAttribute(if run.style.bold {
                Attribute::Bold
            } else {
                Attribute::NormalIntensity
            }),
            SetAttribute(if run.style.reversed {
                Attribute::Reverse
            } else {
                Attribute::NoReverse
            }),
            Print(&run.text)
        )?;
    }

    Ok(())
}
fn read_cell(buffer: &FrameBuffer, x: u16, y: u16) -> Cell {
    if x >= buffer.width() || y >= buffer.height() {
        Cell::default()
    } else {
        buffer.get(x, y)
    }
}

fn map_color(color: Color) -> CrosstermColor {
    match color {
        Color::Reset => CrosstermColor::Reset,
        Color::Black => CrosstermColor::Black,
        Color::DarkGrey => CrosstermColor::DarkGrey,
        Color::Grey => CrosstermColor::Grey,
        Color::White => CrosstermColor::White,
        Color::Blue => CrosstermColor::Blue,
        Color::Cyan => CrosstermColor::Cyan,
        Color::Green => CrosstermColor::Green,
        Color::Yellow => CrosstermColor::Yellow,
        Color::Magenta => CrosstermColor::Magenta,
        Color::Red => CrosstermColor::Red,
        Color::Indexed(index) => CrosstermColor::AnsiValue(index),
        Color::Rgb(r, g, b) => CrosstermColor::Rgb { r, g, b },
    }
}

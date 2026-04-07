use ansiq_core::{Rect, Style, patch_style};
use ansiq_render::{Cell, FrameBuffer};

pub(crate) fn cell(symbol: char, style: Style) -> Cell {
    Cell { symbol, style }
}

pub(crate) fn fill_surface(buffer: &mut FrameBuffer, rect: Rect, style: Style) {
    buffer.fill_rect(rect, ' ', style);
}

pub(crate) fn fill_surface_in_regions(
    buffer: &mut FrameBuffer,
    rect: Rect,
    style: Style,
    regions: &[Rect],
) {
    for region in regions {
        if let Some(clip) = rect.intersection(*region) {
            buffer.fill_rect(clip, ' ', style);
        }
    }
}

pub(crate) fn intersects_any(rect: Rect, regions: &[Rect]) -> bool {
    regions.iter().any(|region| rect.intersects(*region))
}

pub(crate) fn border_style(base: Style, override_style: Style, focused: bool) -> Style {
    let base = patch_style(base, override_style);
    let accent = if focused && !matches!(base.fg, ansiq_core::Color::Reset) {
        base.fg
    } else if focused {
        ansiq_core::Color::Grey
    } else if matches!(base.fg, ansiq_core::Color::Reset) {
        ansiq_core::Color::DarkGrey
    } else {
        base.fg
    };

    base.fg(accent)
}

pub(crate) fn title_style(base: Style, override_style: Style) -> Style {
    let style = patch_style(base, override_style);
    if matches!(style.fg, ansiq_core::Color::Reset) {
        style.fg(ansiq_core::Color::Grey)
    } else {
        style
    }
}

pub(crate) fn single_title(title: Option<&str>) -> Vec<ansiq_core::BlockTitle> {
    title.map(ansiq_core::BlockTitle::new).into_iter().collect()
}

pub(crate) fn merge_highlight_style(base: Style, highlight: Style) -> Style {
    if highlight == Style::default() {
        base.bold(true).reversed(true)
    } else {
        highlight.reversed(true)
    }
}

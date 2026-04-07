use ansiq_core::{Rect, Style};
use unicode_width::UnicodeWidthChar;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub symbol: char,
    pub style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            symbol: ' ',
            style: Style::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FrameBuffer {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

impl FrameBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::default(); usize::from(width) * usize::from(height)],
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn is_blank(&self) -> bool {
        self.cells.iter().all(|cell| *cell == Cell::default())
    }

    pub fn get(&self, x: u16, y: u16) -> Cell {
        self.cells[self.index(x, y)]
    }

    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if x >= self.width || y >= self.height {
            return;
        }

        let index = self.index(x, y);
        self.cells[index] = cell;
    }

    pub fn fill_rect(&mut self, rect: Rect, symbol: char, style: Style) {
        for y in rect.y..rect.bottom().min(self.height) {
            for x in rect.x..rect.right().min(self.width) {
                self.set(x, y, Cell { symbol, style });
            }
        }
    }

    pub fn write_str(&mut self, x: u16, y: u16, text: &str, style: Style) {
        self.write_clipped(
            Rect::new(x, y, self.width.saturating_sub(x), 1),
            0,
            0,
            text,
            style,
        );
    }

    pub fn write_clipped(
        &mut self,
        rect: Rect,
        offset_x: u16,
        offset_y: u16,
        text: &str,
        style: Style,
    ) {
        if rect.width == 0 || rect.height == 0 {
            return;
        }

        let mut cursor_x = rect.x.saturating_add(offset_x);
        let cursor_y = rect.y.saturating_add(offset_y);
        if cursor_y >= rect.bottom() || cursor_y >= self.height {
            return;
        }

        for ch in text.chars() {
            let width = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
            if width == 0 {
                continue;
            }
            if cursor_x >= rect.right() || cursor_x >= self.width {
                break;
            }

            self.set(cursor_x, cursor_y, Cell { symbol: ch, style });

            for fill in 1..width {
                let fill_x = cursor_x.saturating_add(fill);
                if fill_x >= rect.right() || fill_x >= self.width {
                    break;
                }
                self.set(fill_x, cursor_y, Cell { symbol: ' ', style });
            }

            cursor_x = cursor_x.saturating_add(width);
        }
    }

    fn index(&self, x: u16, y: u16) -> usize {
        usize::from(y) * usize::from(self.width) + usize::from(x)
    }
}

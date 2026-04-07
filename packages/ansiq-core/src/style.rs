#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Color {
    #[default]
    Reset,
    Black,
    DarkGrey,
    Grey,
    White,
    Blue,
    Cyan,
    Green,
    Yellow,
    Magenta,
    Red,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub reversed: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reset,
            bold: false,
            reversed: false,
        }
    }
}

impl Style {
    pub const fn fg(mut self, fg: Color) -> Self {
        self.fg = fg;
        self
    }

    pub const fn bg(mut self, bg: Color) -> Self {
        self.bg = bg;
        self
    }

    pub const fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    pub const fn reversed(mut self, reversed: bool) -> Self {
        self.reversed = reversed;
        self
    }
}

impl From<Color> for Style {
    fn from(value: Color) -> Self {
        Style::default().fg(value)
    }
}

pub fn patch_style(base: Style, patch: Style) -> Style {
    let fg = if patch.fg == Color::Reset {
        base.fg
    } else {
        patch.fg
    };
    let bg = if patch.bg == Color::Reset {
        base.bg
    } else {
        patch.bg
    };

    Style {
        fg,
        bg,
        bold: base.bold || patch.bold,
        reversed: base.reversed || patch.reversed,
    }
}

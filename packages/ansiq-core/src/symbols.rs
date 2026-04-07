pub mod scrollbar {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct Set {
        pub track: &'static str,
        pub thumb: &'static str,
        pub begin: &'static str,
        pub end: &'static str,
    }

    pub const DOUBLE_VERTICAL: Set = Set {
        track: "║",
        thumb: "█",
        begin: "▲",
        end: "▼",
    };

    pub const DOUBLE_HORIZONTAL: Set = Set {
        track: "═",
        thumb: "█",
        begin: "◄",
        end: "►",
    };

    pub const VERTICAL: Set = Set {
        track: "│",
        thumb: "█",
        begin: "↑",
        end: "↓",
    };

    pub const HORIZONTAL: Set = Set {
        track: "─",
        thumb: "█",
        begin: "←",
        end: "→",
    };
}

pub mod border {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Set {
        pub top_left: char,
        pub top_right: char,
        pub bottom_left: char,
        pub bottom_right: char,
        pub horizontal_top: char,
        pub horizontal_bottom: char,
        pub vertical_left: char,
        pub vertical_right: char,
    }

    pub const PLAIN: Set = Set {
        top_left: '┌',
        top_right: '┐',
        bottom_left: '└',
        bottom_right: '┘',
        horizontal_top: '─',
        horizontal_bottom: '─',
        vertical_left: '│',
        vertical_right: '│',
    };

    pub const ROUNDED: Set = Set {
        top_left: '╭',
        top_right: '╮',
        bottom_left: '╰',
        bottom_right: '╯',
        horizontal_top: '─',
        horizontal_bottom: '─',
        vertical_left: '│',
        vertical_right: '│',
    };

    pub const DOUBLE: Set = Set {
        top_left: '╔',
        top_right: '╗',
        bottom_left: '╚',
        bottom_right: '╝',
        horizontal_top: '═',
        horizontal_bottom: '═',
        vertical_left: '║',
        vertical_right: '║',
    };

    pub const THICK: Set = Set {
        top_left: '┏',
        top_right: '┓',
        bottom_left: '┗',
        bottom_right: '┛',
        horizontal_top: '━',
        horizontal_bottom: '━',
        vertical_left: '┃',
        vertical_right: '┃',
    };
}

pub mod line {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct Set {
        pub horizontal: &'static str,
    }

    pub const HORIZONTAL: &str = "─";
    pub const VERTICAL: &str = "│";
    pub const DOUBLE_HORIZONTAL: &str = "═";
    pub const THICK_HORIZONTAL: &str = "━";

    pub const NORMAL: Set = Set {
        horizontal: HORIZONTAL,
    };

    pub const DOUBLE: Set = Set {
        horizontal: DOUBLE_HORIZONTAL,
    };

    pub const THICK: Set = Set {
        horizontal: THICK_HORIZONTAL,
    };
}

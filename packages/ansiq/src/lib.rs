pub use ansiq_core::{
    Color, Cx, Element, IntoElement, Rect, Style, ViewCtx, computed, effect, signal,
};
pub use ansiq_macros::view;
pub use ansiq_runtime::{App, RuntimeHandle, run_app, run_app_with_policy};
pub use ansiq_surface::{Viewport, ViewportPolicy};

pub mod core {
    pub use ansiq_core::*;
}

pub mod layout {
    pub use ansiq_layout::*;
}

pub mod render {
    pub use ansiq_render::*;
}

pub mod runtime {
    pub use ansiq_runtime::*;
}

pub mod surface {
    pub use ansiq_surface::*;
}

pub mod widgets {
    pub use ansiq_widgets::*;
}

pub mod prelude {
    pub use crate::core::{ListState, ScrollbarState, TableState};
    pub use crate::widgets::{
        Block, Box, Input, List, Paragraph, ScrollView, Scrollbar, Shell, StatusBar, Table, Tabs,
        Text,
    };
    pub use crate::{
        App, Color, Element, IntoElement, Rect, RuntimeHandle, Style, ViewCtx, computed, effect,
        signal,
    };
}

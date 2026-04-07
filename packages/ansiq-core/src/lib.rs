mod element;
mod geometry;
mod history;
mod hooks;
mod reactivity;
mod render_math;
mod style;
pub mod symbols;
mod table;
mod text;
mod transcript;

pub use element::{
    Alignment, Bar, BarChartProps, BlockFrame, BlockProps, BlockTitle, BorderType, Borders,
    BoxProps, CanvasCell, CanvasProps, ChangeHandler, ChartDataset, ChartProps, ChildLayoutKind,
    ChildLayoutSpec, ClearProps, ComponentProps, ComponentRenderer, Constraint, Direction, Element,
    ElementKind, Flex, GaugeProps, HighlightSpacing, InputProps, IntoElement, Layout, Length,
    LineGaugeProps, ListDirection, ListItem, ListProps, ListState, MonthlyProps, Node, Padding,
    PaneProps, ParagraphProps, RichTextProps, RuntimeWidgetState, ScrollDirection, ScrollHandler,
    ScrollViewProps, ScrollbarOrientation, ScrollbarProps, ScrollbarState, SelectHandler,
    ShellProps, SparklineDirection, SparklineProps, StatusBarProps, StreamingTextProps,
    SubmitHandler, TableAlignment, TableProps, TableState, TabsProps, TextProps, TitlePosition,
    WidgetKey, WidgetRouteContext, WidgetRouteEffect, Wrap, component, component_with_cx,
};
pub use geometry::Rect;
pub use history::{HistoryBlock, HistoryEntry, HistoryLine, HistoryRun, history_block_from_text};
pub use hooks::{Cx, HookStore, RuntimeRequest, ViewCtx};
pub use reactivity::{
    Computed, EffectHandle, ScopeId, Signal, SignalId, computed, current_reactive_scope,
    dispose_component_scope, effect, flush_reactivity, render_in_component_scope,
    reset_reactivity_for_testing, signal, take_dirty_component_scopes,
};
pub use render_math::{
    TitleGroupPositions, table_column_layout, table_span_width, title_group_positions,
};
pub use style::{Color, Style, patch_style};
pub use table::{Cell, Row};
pub use text::{
    Line, Span, StyledChunk, StyledLine, Text, clip_to_width, display_width, display_width_prefix,
    styled_line_from_line, styled_line_from_span, styled_lines_from_text, wrap_plain_lines,
    wrap_styled_lines,
};
pub use transcript::{TranscriptEntry, TranscriptRole, TranscriptSession, transcript_block};

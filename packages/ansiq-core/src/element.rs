use std::{
    fmt,
    ops::{BitOr, BitOrAssign},
    rc::Rc,
};

use crate::{HistoryBlock, Line, Rect, Row, ScopeId, Span, Style, Text};
use unicode_width::UnicodeWidthChar;

pub type ChangeHandler = std::boxed::Box<dyn FnMut(String)>;
pub type SubmitHandler<Message> = std::boxed::Box<dyn FnMut(String) -> Option<Message>>;
pub type SelectHandler<Message> = std::boxed::Box<dyn FnMut(usize) -> Option<Message>>;
pub type ScrollHandler<Message> = std::boxed::Box<dyn FnMut(usize) -> Option<Message>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeWidgetState {
    InputCursor(usize),
    List(ListState),
    Tabs(Option<usize>),
    Table(TableState),
    ScrollView(Option<usize>),
    Scrollbar(ScrollbarState),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidgetKey {
    Up,
    Down,
    Left,
    Right,
    Escape,
    Enter,
    Backspace,
    Char(char),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WidgetRouteContext {
    pub viewport_height: usize,
    pub scroll_view_max_offset: Option<usize>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct WidgetRouteEffect<Message> {
    pub dirty: bool,
    pub message: Option<Message>,
}

impl<Message> Default for WidgetRouteEffect<Message> {
    fn default() -> Self {
        Self {
            dirty: false,
            message: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    Row,
    #[default]
    Column,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Length {
    #[default]
    Auto,
    Fill,
    Fixed(u16),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Constraint {
    Length(u16),
    Percentage(u16),
    Fill(u16),
    Min(u16),
    Max(u16),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Flex {
    Legacy,
    #[default]
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl From<u16> for Constraint {
    fn from(value: u16) -> Self {
        Self::Length(value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Wrap {
    pub trim: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Padding {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

impl Padding {
    pub const fn zero() -> Self {
        Self {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        }
    }

    pub const fn all(value: u16) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn symmetric(horizontal: u16, vertical: u16) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    pub width: Length,
    pub height: Length,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            width: Length::Fill,
            height: Length::Auto,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChildLayoutKind {
    Fill,
    Stack { direction: Direction, gap: u16 },
    Shell,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChildLayoutSpec {
    pub bounds: Rect,
    pub kind: ChildLayoutKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoxProps {
    pub direction: Direction,
    pub gap: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextProps {
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaneProps {
    pub title: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Borders(u8);

impl Borders {
    pub const NONE: Self = Self(0);
    pub const TOP: Self = Self(0b0001);
    pub const RIGHT: Self = Self(0b0010);
    pub const BOTTOM: Self = Self(0b0100);
    pub const LEFT: Self = Self(0b1000);
    pub const ALL: Self = Self(Self::TOP.0 | Self::RIGHT.0 | Self::BOTTOM.0 | Self::LEFT.0);

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl BitOr for Borders {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Borders {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BorderType {
    #[default]
    Plain,
    Rounded,
    Double,
    Thick,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TitlePosition {
    #[default]
    Top,
    Bottom,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockTitle {
    pub position: Option<TitlePosition>,
    pub content: Line,
}

impl BlockTitle {
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Line>,
    {
        Self {
            position: None,
            content: content.into(),
        }
    }

    pub fn top<T>(content: T) -> Self
    where
        T: Into<Line>,
    {
        Self {
            position: Some(TitlePosition::Top),
            content: content.into(),
        }
    }

    pub fn bottom<T>(content: T) -> Self
    where
        T: Into<Line>,
    {
        Self {
            position: Some(TitlePosition::Bottom),
            content: content.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockProps {
    pub titles: Vec<BlockTitle>,
    pub title_alignment: Alignment,
    pub title_position: TitlePosition,
    pub borders: Borders,
    pub border_type: BorderType,
    pub border_set: Option<crate::symbols::border::Set>,
    pub padding: Padding,
    pub border_style: Style,
    pub title_style: Style,
}

impl BlockProps {
    pub fn has_title_at_position(&self, position: TitlePosition) -> bool {
        self.titles
            .iter()
            .any(|title| title.position.unwrap_or(self.title_position) == position)
    }

    pub fn inner(&self, rect: Rect) -> Rect {
        let left = u16::from(self.borders.contains(Borders::LEFT));
        let right = u16::from(self.borders.contains(Borders::RIGHT));
        let top = u16::from(
            self.borders.contains(Borders::TOP) || self.has_title_at_position(TitlePosition::Top),
        );
        let bottom = u16::from(
            self.borders.contains(Borders::BOTTOM)
                || self.has_title_at_position(TitlePosition::Bottom),
        );

        Rect::new(
            rect.x.saturating_add(left),
            rect.y.saturating_add(top),
            rect.width.saturating_sub(left.saturating_add(right)),
            rect.height.saturating_sub(top.saturating_add(bottom)),
        )
        .inset(self.padding)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockFrame {
    pub props: BlockProps,
    pub style: Style,
}

impl BlockFrame {
    pub fn inner(&self, rect: Rect) -> Rect {
        self.props.inner(rect)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParagraphProps {
    pub content: Text,
    pub block: Option<BlockFrame>,
    pub alignment: Alignment,
    pub wrap: Option<Wrap>,
    pub scroll_x: u16,
    pub scroll_y: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RichTextProps {
    pub block: HistoryBlock,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HighlightSpacing {
    Always,
    #[default]
    WhenSelected,
    Never,
}

impl HighlightSpacing {
    pub const fn should_add(&self, has_selection: bool) -> bool {
        match self {
            Self::Always => true,
            Self::WhenSelected => has_selection,
            Self::Never => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListDirection {
    #[default]
    TopToBottom,
    BottomToTop,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ListItem {
    pub content: Text,
    pub style: Style,
}

impl ListItem {
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Text>,
    {
        Self {
            content: content.into(),
            style: Style::default(),
        }
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    pub fn height(&self) -> usize {
        self.content.height()
    }

    pub fn width(&self) -> usize {
        self.content.width()
    }
}

impl<T> From<T> for ListItem
where
    T: Into<Text>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ListState {
    offset: usize,
    selected: Option<usize>,
}

impl ListState {
    pub const fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    pub const fn with_selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }

    pub const fn offset_mut(&mut self) -> &mut usize {
        &mut self.offset
    }

    pub const fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub const fn selected_mut(&mut self) -> &mut Option<usize> {
        &mut self.selected
    }

    pub const fn select(&mut self, index: Option<usize>) {
        self.selected = index;
        if index.is_none() {
            self.offset = 0;
        }
    }

    pub fn select_next(&mut self) {
        let next = self.selected.map_or(0, |i| i.saturating_add(1));
        self.select(Some(next));
    }

    pub fn select_previous(&mut self) {
        let previous = self.selected.map_or(usize::MAX, |i| i.saturating_sub(1));
        self.select(Some(previous));
    }

    pub const fn select_first(&mut self) {
        self.select(Some(0));
    }

    pub const fn select_last(&mut self) {
        self.select(Some(usize::MAX));
    }

    pub fn scroll_down_by(&mut self, amount: u16) {
        let selected = self.selected.unwrap_or_default();
        self.select(Some(selected.saturating_add(amount as usize)));
    }

    pub fn scroll_up_by(&mut self, amount: u16) {
        let selected = self.selected.unwrap_or_default();
        self.select(Some(selected.saturating_sub(amount as usize)));
    }
}

pub struct ListProps<Message> {
    pub block: Option<BlockFrame>,
    pub items: Vec<ListItem>,
    pub state: ListState,
    pub highlight_symbol: Option<Line>,
    pub highlight_style: Style,
    pub highlight_spacing: HighlightSpacing,
    pub repeat_highlight_symbol: bool,
    pub direction: ListDirection,
    pub scroll_padding: usize,
    pub on_select: Option<SelectHandler<Message>>,
}

impl<Message> fmt::Debug for ListProps<Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ListProps")
            .field("block", &self.block)
            .field("items", &self.items)
            .field("state", &self.state)
            .field("highlight_symbol", &self.highlight_symbol)
            .field("highlight_style", &self.highlight_style)
            .field("highlight_spacing", &self.highlight_spacing)
            .field("repeat_highlight_symbol", &self.repeat_highlight_symbol)
            .field("direction", &self.direction)
            .field("scroll_padding", &self.scroll_padding)
            .finish()
    }
}

pub struct TabsProps<Message> {
    pub block: Option<BlockFrame>,
    pub titles: Vec<Line>,
    pub selected: Option<usize>,
    pub selection_explicit: bool,
    pub highlight_style: Style,
    pub divider: Span,
    pub padding_left: Line,
    pub padding_right: Line,
    pub on_select: Option<SelectHandler<Message>>,
}

impl<Message> fmt::Debug for TabsProps<Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TabsProps")
            .field("block", &self.block)
            .field("titles", &self.titles)
            .field("selected", &self.selected)
            .field("selection_explicit", &self.selection_explicit)
            .field("highlight_style", &self.highlight_style)
            .field("divider", &self.divider)
            .field("padding_left", &self.padding_left)
            .field("padding_right", &self.padding_right)
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GaugeProps {
    pub block: Option<BlockFrame>,
    pub ratio: f64,
    pub label: Option<Span>,
    pub use_unicode: bool,
    pub gauge_style: Style,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClearProps;

#[derive(Clone, Debug, PartialEq)]
pub struct LineGaugeProps {
    pub block: Option<BlockFrame>,
    pub ratio: f64,
    pub label: Option<Line>,
    pub filled_symbol: String,
    pub unfilled_symbol: String,
    pub filled_style: Style,
    pub unfilled_style: Style,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TableAlignment {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TableState {
    offset: usize,
    selected: Option<usize>,
    selected_column: Option<usize>,
}

impl TableState {
    pub const fn new() -> Self {
        Self {
            offset: 0,
            selected: None,
            selected_column: None,
        }
    }

    pub const fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_selected(mut self, selected: impl Into<Option<usize>>) -> Self {
        self.selected = selected.into();
        self
    }

    pub fn with_selected_column(mut self, selected: impl Into<Option<usize>>) -> Self {
        self.selected_column = selected.into();
        self
    }

    pub fn with_selected_cell(mut self, selected: impl Into<Option<(usize, usize)>>) -> Self {
        if let Some((row, column)) = selected.into() {
            self.selected = Some(row);
            self.selected_column = Some(column);
        } else {
            self.selected = None;
            self.selected_column = None;
        }
        self
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }

    pub const fn offset_mut(&mut self) -> &mut usize {
        &mut self.offset
    }

    pub const fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub const fn selected_column(&self) -> Option<usize> {
        self.selected_column
    }

    pub const fn selected_cell(&self) -> Option<(usize, usize)> {
        if let (Some(row), Some(column)) = (self.selected, self.selected_column) {
            Some((row, column))
        } else {
            None
        }
    }

    pub const fn selected_mut(&mut self) -> &mut Option<usize> {
        &mut self.selected
    }

    pub const fn selected_column_mut(&mut self) -> &mut Option<usize> {
        &mut self.selected_column
    }

    pub const fn select(&mut self, index: Option<usize>) {
        self.selected = index;
        if index.is_none() {
            self.offset = 0;
        }
    }

    pub const fn select_column(&mut self, index: Option<usize>) {
        self.selected_column = index;
    }

    pub const fn select_cell(&mut self, indexes: Option<(usize, usize)>) {
        if let Some((row, column)) = indexes {
            self.selected = Some(row);
            self.selected_column = Some(column);
        } else {
            self.offset = 0;
            self.selected = None;
            self.selected_column = None;
        }
    }

    pub fn select_next(&mut self) {
        let next = self.selected.map_or(0, |i| i.saturating_add(1));
        self.select(Some(next));
    }

    pub fn select_next_column(&mut self) {
        let next = self.selected_column.map_or(0, |i| i.saturating_add(1));
        self.select_column(Some(next));
    }

    pub fn select_previous(&mut self) {
        let previous = self.selected.map_or(usize::MAX, |i| i.saturating_sub(1));
        self.select(Some(previous));
    }

    pub fn select_previous_column(&mut self) {
        let previous = self
            .selected_column
            .map_or(usize::MAX, |i| i.saturating_sub(1));
        self.select_column(Some(previous));
    }

    pub const fn select_first(&mut self) {
        self.select(Some(0));
    }

    pub const fn select_first_column(&mut self) {
        self.select_column(Some(0));
    }

    pub const fn select_last(&mut self) {
        self.select(Some(usize::MAX));
    }

    pub const fn select_last_column(&mut self) {
        self.select_column(Some(usize::MAX));
    }

    pub fn scroll_down_by(&mut self, amount: u16) {
        let selected = self.selected.unwrap_or_default();
        self.select(Some(selected.saturating_add(amount as usize)));
    }

    pub fn scroll_up_by(&mut self, amount: u16) {
        let selected = self.selected.unwrap_or_default();
        self.select(Some(selected.saturating_sub(amount as usize)));
    }

    pub fn scroll_right_by(&mut self, amount: u16) {
        let selected = self.selected_column.unwrap_or_default();
        self.select_column(Some(selected.saturating_add(amount as usize)));
    }

    pub fn scroll_left_by(&mut self, amount: u16) {
        let selected = self.selected_column.unwrap_or_default();
        self.select_column(Some(selected.saturating_sub(amount as usize)));
    }
}

pub struct TableProps<Message> {
    pub block: Option<BlockFrame>,
    pub header: Option<Row>,
    pub footer: Option<Row>,
    pub rows: Vec<Row>,
    pub widths: Vec<Constraint>,
    pub column_spacing: u16,
    pub flex: Flex,
    pub alignments: Vec<TableAlignment>,
    pub state: TableState,
    pub highlight_symbol: Option<Text>,
    pub row_highlight_style: Style,
    pub column_highlight_style: Style,
    pub cell_highlight_style: Style,
    pub highlight_spacing: HighlightSpacing,
    pub on_select: Option<SelectHandler<Message>>,
}

impl<Message> fmt::Debug for TableProps<Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TableProps")
            .field("block", &self.block)
            .field("header", &self.header)
            .field("footer", &self.footer)
            .field("rows", &self.rows)
            .field("widths", &self.widths)
            .field("column_spacing", &self.column_spacing)
            .field("flex", &self.flex)
            .field("alignments", &self.alignments)
            .field("state", &self.state)
            .field("highlight_symbol", &self.highlight_symbol)
            .field("row_highlight_style", &self.row_highlight_style)
            .field("column_highlight_style", &self.column_highlight_style)
            .field("cell_highlight_style", &self.cell_highlight_style)
            .field("highlight_spacing", &self.highlight_spacing)
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SparklineDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SparklineProps {
    pub values: Vec<Option<u64>>,
    pub max: Option<u64>,
    pub direction: SparklineDirection,
    pub absent_value_symbol: char,
    pub absent_value_style: Style,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bar {
    pub label: String,
    pub value: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BarChartProps {
    pub bars: Vec<Bar>,
    pub max: Option<u64>,
    pub bar_width: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChartDataset {
    pub label: Option<String>,
    pub points: Vec<(i64, i64)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChartProps {
    pub datasets: Vec<ChartDataset>,
    pub min_y: Option<i64>,
    pub max_y: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasCell {
    pub x: u16,
    pub y: u16,
    pub symbol: char,
    pub style: Style,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasProps {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<CanvasCell>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonthlyProps {
    pub year: i32,
    pub month: u8,
    pub selected_day: Option<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ScrollbarOrientation {
    #[default]
    VerticalRight,
    VerticalLeft,
    HorizontalBottom,
    HorizontalTop,
}

impl ScrollbarOrientation {
    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::VerticalRight | Self::VerticalLeft)
    }

    pub const fn is_horizontal(self) -> bool {
        matches!(self, Self::HorizontalBottom | Self::HorizontalTop)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ScrollDirection {
    #[default]
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ScrollbarState {
    content_length: usize,
    position: usize,
    viewport_content_length: usize,
}

impl ScrollbarState {
    pub const fn new(content_length: usize) -> Self {
        Self {
            content_length,
            position: 0,
            viewport_content_length: 0,
        }
    }

    pub const fn content_length(mut self, content_length: usize) -> Self {
        self.content_length = content_length;
        self
    }

    pub const fn content_length_value(&self) -> usize {
        self.content_length
    }

    pub const fn get_position(&self) -> usize {
        self.position
    }

    pub const fn viewport_content_length_value(&self) -> usize {
        self.viewport_content_length
    }

    pub const fn with_content_length(mut self, content_length: usize) -> Self {
        self.content_length = content_length;
        self
    }

    pub const fn with_position(mut self, position: usize) -> Self {
        self.position = position;
        self
    }

    pub const fn with_viewport_content_length(mut self, viewport_content_length: usize) -> Self {
        self.viewport_content_length = viewport_content_length;
        self
    }

    pub const fn position(mut self, position: usize) -> Self {
        self.position = position;
        self
    }

    pub const fn viewport_content_length(mut self, viewport_content_length: usize) -> Self {
        self.viewport_content_length = viewport_content_length;
        self
    }

    pub const fn content_length_mut(&mut self) -> &mut usize {
        &mut self.content_length
    }

    pub const fn position_mut(&mut self) -> &mut usize {
        &mut self.position
    }

    pub const fn viewport_content_length_mut(&mut self) -> &mut usize {
        &mut self.viewport_content_length
    }

    pub const fn prev(&mut self) {
        self.position = self.position.saturating_sub(1);
    }

    pub fn next(&mut self) {
        self.position = self
            .position
            .saturating_add(1)
            .min(self.content_length.saturating_sub(1));
    }

    pub const fn first(&mut self) {
        self.position = 0;
    }

    pub const fn last(&mut self) {
        self.position = self.content_length.saturating_sub(1);
    }

    pub fn scroll(&mut self, direction: ScrollDirection) {
        match direction {
            ScrollDirection::Forward => self.next(),
            ScrollDirection::Backward => self.prev(),
        }
    }
}

pub struct ScrollViewProps<Message> {
    pub follow_bottom: bool,
    pub offset: Option<usize>,
    pub on_scroll: Option<ScrollHandler<Message>>,
}

impl<Message> fmt::Debug for ScrollViewProps<Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScrollViewProps")
            .field("follow_bottom", &self.follow_bottom)
            .field("offset", &self.offset)
            .finish()
    }
}

pub struct ScrollbarProps<Message> {
    pub orientation: ScrollbarOrientation,
    pub state: ScrollbarState,
    pub thumb_symbol: String,
    pub thumb_style: Style,
    pub track_symbol: Option<String>,
    pub track_style: Style,
    pub begin_symbol: Option<String>,
    pub begin_style: Style,
    pub end_symbol: Option<String>,
    pub end_style: Style,
    pub on_scroll: Option<ScrollHandler<Message>>,
}

impl<Message> fmt::Debug for ScrollbarProps<Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScrollbarProps")
            .field("orientation", &self.orientation)
            .field("state", &self.state)
            .field("thumb_symbol", &self.thumb_symbol)
            .field("track_symbol", &self.track_symbol)
            .field("begin_symbol", &self.begin_symbol)
            .field("end_symbol", &self.end_symbol)
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamingTextProps {
    pub content: String,
}

pub struct InputProps<Message> {
    pub value: String,
    pub placeholder: String,
    pub on_change: Option<ChangeHandler>,
    pub on_submit: Option<SubmitHandler<Message>>,
    pub cursor: usize,
}

impl<Message> fmt::Debug for InputProps<Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InputProps")
            .field("value", &self.value)
            .field("placeholder", &self.placeholder)
            .field("cursor", &self.cursor)
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusBarProps {
    pub content: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShellProps;

pub struct ComponentProps<Message> {
    pub name: String,
    pub scope: Option<ScopeId>,
    pub renderer: ComponentRenderer<Message>,
}

impl<Message> fmt::Debug for ComponentProps<Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComponentProps")
            .field("name", &self.name)
            .field("scope", &self.scope)
            .finish()
    }
}

pub enum ComponentRenderer<Message> {
    Static(Rc<dyn Fn() -> Element<Message>>),
    WithCx(Rc<dyn for<'a> Fn(&mut crate::ViewCtx<'a, Message>) -> Element<Message>>),
}

impl<Message> Clone for ComponentRenderer<Message> {
    fn clone(&self) -> Self {
        match self {
            Self::Static(renderer) => Self::Static(renderer.clone()),
            Self::WithCx(renderer) => Self::WithCx(renderer.clone()),
        }
    }
}

pub enum ElementKind<Message> {
    Box(BoxProps),
    Text(TextProps),
    Pane(PaneProps),
    Block(BlockProps),
    Paragraph(ParagraphProps),
    RichText(RichTextProps),
    List(ListProps<Message>),
    Tabs(TabsProps<Message>),
    Gauge(GaugeProps),
    Clear(ClearProps),
    LineGauge(LineGaugeProps),
    Table(TableProps<Message>),
    Sparkline(SparklineProps),
    BarChart(BarChartProps),
    Chart(ChartProps),
    Canvas(CanvasProps),
    Monthly(MonthlyProps),
    ScrollView(ScrollViewProps<Message>),
    Scrollbar(ScrollbarProps<Message>),
    StreamingText(StreamingTextProps),
    Input(InputProps<Message>),
    StatusBar(StatusBarProps),
    Shell(ShellProps),
    Component(ComponentProps<Message>),
}

impl<Message> ElementKind<Message> {
    pub fn child_layout_spec(&self, bounds: Rect) -> ChildLayoutSpec {
        match self {
            Self::Box(props) => ChildLayoutSpec {
                bounds,
                kind: ChildLayoutKind::Stack {
                    direction: props.direction,
                    gap: props.gap,
                },
            },
            Self::Shell(_) => ChildLayoutSpec {
                bounds,
                kind: ChildLayoutKind::Shell,
            },
            Self::Pane(_) => ChildLayoutSpec {
                bounds: bounds.shrink(1),
                kind: ChildLayoutKind::Fill,
            },
            Self::Block(props) => ChildLayoutSpec {
                bounds: props.inner(bounds),
                kind: ChildLayoutKind::Stack {
                    direction: Direction::Column,
                    gap: 0,
                },
            },
            Self::ScrollView(_) | Self::Component(_) => ChildLayoutSpec {
                bounds,
                kind: ChildLayoutKind::Fill,
            },
            _ => ChildLayoutSpec {
                bounds,
                kind: ChildLayoutKind::Fill,
            },
        }
    }

    pub fn capture_runtime_state(&self) -> Option<RuntimeWidgetState> {
        match self {
            Self::Input(props) => Some(RuntimeWidgetState::InputCursor(
                props.cursor.min(props.value.chars().count()),
            )),
            Self::List(props) => Some(RuntimeWidgetState::List(props.state)),
            Self::Tabs(props) => Some(RuntimeWidgetState::Tabs(props.selected)),
            Self::Table(props) => Some(RuntimeWidgetState::Table(props.state)),
            Self::ScrollView(props) => Some(RuntimeWidgetState::ScrollView(props.offset)),
            Self::Scrollbar(props) => Some(RuntimeWidgetState::Scrollbar(props.state)),
            _ => None,
        }
    }

    pub fn restore_runtime_state(&mut self, state: &RuntimeWidgetState) -> bool {
        match (self, state) {
            (Self::Input(props), RuntimeWidgetState::InputCursor(cursor)) => {
                let max = props.value.chars().count();
                props.cursor = (*cursor).min(max);
                true
            }
            (Self::List(props), RuntimeWidgetState::List(previous))
                if props.state == ListState::default() && *previous != ListState::default() =>
            {
                props.state = *previous;
                true
            }
            (Self::Tabs(props), RuntimeWidgetState::Tabs(previous))
                if !props.selection_explicit && props.selected != *previous =>
            {
                props.selected = *previous;
                true
            }
            (Self::Table(props), RuntimeWidgetState::Table(previous))
                if props.state == TableState::default() && *previous != TableState::default() =>
            {
                props.state = *previous;
                true
            }
            (Self::ScrollView(props), RuntimeWidgetState::ScrollView(previous))
                if props.offset.is_none() && previous.is_some() =>
            {
                props.offset = *previous;
                true
            }
            (Self::Scrollbar(props), RuntimeWidgetState::Scrollbar(previous))
                if props.state.get_position() == 0 && previous.get_position() > 0 =>
            {
                props.state = *previous;
                true
            }
            _ => false,
        }
    }

    pub fn initialize_runtime_state(&mut self) {
        match self {
            Self::Input(props) => {
                props.cursor = props.cursor.min(props.value.chars().count());
                if props.cursor == 0 {
                    props.cursor = props.value.chars().count();
                }
            }
            Self::Tabs(props) => {
                if props.selected.is_none() && !props.titles.is_empty() {
                    props.selected = Some(0);
                }
            }
            _ => {}
        }
    }

    pub fn intrinsic_height(&self, width: u16, child_heights: &[u16]) -> u16 {
        match self {
            Self::Box(props) => match props.direction {
                Direction::Column => {
                    let gaps = props
                        .gap
                        .saturating_mul(child_heights.len().saturating_sub(1) as u16);
                    child_heights
                        .iter()
                        .copied()
                        .fold(0u16, u16::saturating_add)
                        .saturating_add(gaps)
                }
                Direction::Row => child_heights.iter().copied().max().unwrap_or(1),
            },
            Self::Pane(_) => child_heights
                .iter()
                .copied()
                .max()
                .unwrap_or(0)
                .saturating_add(2),
            Self::Block(props) => child_heights
                .iter()
                .copied()
                .fold(0u16, u16::saturating_add)
                .saturating_add(props.padding.top)
                .saturating_add(props.padding.bottom)
                .saturating_add(block_vertical_inset(props))
                .max(1),
            Self::Shell(_) => child_heights
                .iter()
                .copied()
                .fold(0u16, u16::saturating_add)
                .max(1),
            Self::ScrollView(_) | Self::Component(_) => child_heights.first().copied().unwrap_or(1),
            Self::Text(props) => wrapped_line_count(&props.content, width),
            Self::Paragraph(props) => paragraph_height(props, width),
            Self::RichText(props) => props.block.lines.len().max(1) as u16,
            Self::StreamingText(props) => wrapped_line_count(&props.content, width),
            Self::List(props) => list_height(props),
            Self::Tabs(props) => tabs_height(props),
            Self::Gauge(props) => block_height(props.block.as_ref(), 1),
            Self::LineGauge(props) => block_height(props.block.as_ref(), 1),
            Self::Sparkline(_) => 1,
            Self::BarChart(_) => 6,
            Self::Chart(_) | Self::Canvas(_) | Self::Monthly(_) => 8,
            Self::Clear(_) => 1,
            Self::Table(props) => table_height(props),
            Self::Scrollbar(_) => 1,
            Self::StatusBar(props) => wrapped_line_count(&props.content, width),
            Self::Input(_) => 3,
        }
    }

    pub fn intrinsic_width(&self, child_widths: &[u16]) -> u16 {
        match self {
            Self::Box(props) => match props.direction {
                Direction::Column => child_widths.iter().copied().max().unwrap_or(1),
                Direction::Row => {
                    let gaps = props
                        .gap
                        .saturating_mul(child_widths.len().saturating_sub(1) as u16);
                    child_widths
                        .iter()
                        .copied()
                        .fold(0u16, u16::saturating_add)
                        .saturating_add(gaps)
                        .max(1)
                }
            },
            Self::Pane(props) => child_widths
                .iter()
                .copied()
                .max()
                .unwrap_or(0)
                .max(title_width(props.title.as_deref()))
                .saturating_add(2)
                .max(1),
            Self::Block(props) => child_widths
                .iter()
                .copied()
                .max()
                .unwrap_or(0)
                .max(block_title_width(props))
                .saturating_add(props.padding.left)
                .saturating_add(props.padding.right)
                .saturating_add(border_horizontal_inset(props.borders))
                .max(1),
            Self::Shell(_) => child_widths.iter().copied().max().unwrap_or(1),
            Self::ScrollView(_) | Self::Component(_) => child_widths.first().copied().unwrap_or(1),
            Self::Text(props) => plain_text_width(&props.content).max(1),
            Self::Paragraph(props) => paragraph_width(props).max(1),
            Self::RichText(props) => rich_text_width(&props.block).max(1),
            Self::StreamingText(props) => plain_text_width(&props.content).max(1),
            Self::List(props) => list_width(props).max(1),
            Self::Tabs(props) => tabs_width(props).max(1),
            Self::Gauge(props) => block_width(
                props.block.as_ref(),
                props.label.as_ref().map_or(1, |label| label.width() as u16),
            ),
            Self::LineGauge(props) => block_width(
                props.block.as_ref(),
                props.label.as_ref().map_or(1, |label| label.width() as u16),
            ),
            Self::Sparkline(props) => props.values.len().max(1) as u16,
            Self::BarChart(props) => {
                let bars = props.bars.len() as u16;
                bars.saturating_mul(props.bar_width.max(1))
                    .saturating_add(bars.saturating_sub(1))
                    .max(1)
            }
            Self::Chart(_) | Self::Canvas(_) | Self::Monthly(_) => 8,
            Self::Clear(_) => 1,
            Self::Table(props) => table_width(props).max(1),
            Self::Scrollbar(props) => {
                if props.orientation.is_vertical() {
                    1
                } else {
                    props
                        .state
                        .viewport_content_length_value()
                        .max(1)
                        .try_into()
                        .unwrap_or(u16::MAX)
                }
            }
            Self::StatusBar(props) => plain_text_width(&props.content).max(1),
            Self::Input(props) => plain_text_width(if props.value.is_empty() {
                &props.placeholder
            } else {
                &props.value
            })
            .saturating_add(2)
            .max(1),
        }
    }

    pub fn invalidates_self_on_layout_change(&self, style: Style) -> bool {
        match self {
            Self::Box(_) | Self::Shell(_) | Self::ScrollView(_) => style != Style::default(),
            Self::Component(_) => false,
            _ => true,
        }
    }

    pub fn route_widget_key(
        &mut self,
        key: WidgetKey,
        context: WidgetRouteContext,
    ) -> Option<WidgetRouteEffect<Message>> {
        match self {
            Self::List(props) => {
                match key {
                    WidgetKey::Up => step_list_selection(&mut props.state, props.items.len(), -1)
                        .map(|index| {
                            sync_list_offset(props, context.viewport_height);
                            WidgetRouteEffect {
                                dirty: true,
                                message: props
                                    .on_select
                                    .as_mut()
                                    .and_then(|on_select| on_select(index)),
                            }
                        }),
                    WidgetKey::Down => step_list_selection(&mut props.state, props.items.len(), 1)
                        .map(|index| {
                            sync_list_offset(props, context.viewport_height);
                            WidgetRouteEffect {
                                dirty: true,
                                message: props
                                    .on_select
                                    .as_mut()
                                    .and_then(|on_select| on_select(index)),
                            }
                        }),
                    WidgetKey::Enter => Some(WidgetRouteEffect {
                        message: props.state.selected().and_then(|index| {
                            props
                                .on_select
                                .as_mut()
                                .and_then(|on_select| on_select(index))
                        }),
                        ..WidgetRouteEffect::default()
                    }),
                    _ => None,
                }
            }
            Self::Tabs(props) => match key {
                WidgetKey::Left => step_selected_index(&mut props.selected, props.titles.len(), -1)
                    .map(|index| WidgetRouteEffect {
                        dirty: true,
                        message: props
                            .on_select
                            .as_mut()
                            .and_then(|on_select| on_select(index)),
                    }),
                WidgetKey::Right => step_selected_index(&mut props.selected, props.titles.len(), 1)
                    .map(|index| WidgetRouteEffect {
                        dirty: true,
                        message: props
                            .on_select
                            .as_mut()
                            .and_then(|on_select| on_select(index)),
                    }),
                WidgetKey::Enter => Some(WidgetRouteEffect {
                    message: props.selected.and_then(|selected| {
                        props
                            .on_select
                            .as_mut()
                            .and_then(|on_select| on_select(selected))
                    }),
                    ..WidgetRouteEffect::default()
                }),
                _ => None,
            },
            Self::Table(props) => {
                match key {
                    WidgetKey::Up => step_table_selection(&mut props.state, props.rows.len(), -1)
                        .map(|index| {
                            sync_table_offset(props, context.viewport_height);
                            WidgetRouteEffect {
                                dirty: true,
                                message: props
                                    .on_select
                                    .as_mut()
                                    .and_then(|on_select| on_select(index)),
                            }
                        }),
                    WidgetKey::Down => step_table_selection(&mut props.state, props.rows.len(), 1)
                        .map(|index| {
                            sync_table_offset(props, context.viewport_height);
                            WidgetRouteEffect {
                                dirty: true,
                                message: props
                                    .on_select
                                    .as_mut()
                                    .and_then(|on_select| on_select(index)),
                            }
                        }),
                    WidgetKey::Enter => Some(WidgetRouteEffect {
                        message: props.state.selected().and_then(|index| {
                            props
                                .on_select
                                .as_mut()
                                .and_then(|on_select| on_select(index))
                        }),
                        ..WidgetRouteEffect::default()
                    }),
                    _ => None,
                }
            }
            Self::ScrollView(props) => {
                let max_offset = context.scroll_view_max_offset?;
                let current =
                    props
                        .offset
                        .unwrap_or(if props.follow_bottom { max_offset } else { 0 });
                let next = match key {
                    WidgetKey::Up => current.saturating_sub(1),
                    WidgetKey::Down => current.saturating_add(1).min(max_offset),
                    WidgetKey::Enter => current,
                    _ => return None,
                };

                if matches!(key, WidgetKey::Enter) {
                    return Some(WidgetRouteEffect {
                        message: props
                            .on_scroll
                            .as_mut()
                            .and_then(|on_scroll| on_scroll(current)),
                        ..WidgetRouteEffect::default()
                    });
                }

                if next == current {
                    return Some(WidgetRouteEffect::default());
                }

                props.offset = if props.follow_bottom && next == max_offset {
                    None
                } else {
                    Some(next)
                };

                Some(WidgetRouteEffect {
                    dirty: true,
                    message: props
                        .on_scroll
                        .as_mut()
                        .and_then(|on_scroll| on_scroll(next)),
                })
            }
            Self::Scrollbar(props) => {
                let max_position = props
                    .state
                    .content_length_value()
                    .saturating_sub(props.state.viewport_content_length_value());
                let current = props.state.get_position().min(max_position);
                let next = match key {
                    WidgetKey::Up
                        if matches!(
                            props.orientation,
                            ScrollbarOrientation::VerticalRight
                                | ScrollbarOrientation::VerticalLeft
                        ) =>
                    {
                        current.saturating_sub(1)
                    }
                    WidgetKey::Down
                        if matches!(
                            props.orientation,
                            ScrollbarOrientation::VerticalRight
                                | ScrollbarOrientation::VerticalLeft
                        ) =>
                    {
                        current.saturating_add(1).min(max_position)
                    }
                    WidgetKey::Left
                        if matches!(
                            props.orientation,
                            ScrollbarOrientation::HorizontalBottom
                                | ScrollbarOrientation::HorizontalTop
                        ) =>
                    {
                        current.saturating_sub(1)
                    }
                    WidgetKey::Right
                        if matches!(
                            props.orientation,
                            ScrollbarOrientation::HorizontalBottom
                                | ScrollbarOrientation::HorizontalTop
                        ) =>
                    {
                        current.saturating_add(1).min(max_position)
                    }
                    WidgetKey::Enter => current,
                    _ => return None,
                };

                if matches!(key, WidgetKey::Enter) {
                    return Some(WidgetRouteEffect {
                        message: props
                            .on_scroll
                            .as_mut()
                            .and_then(|on_scroll| on_scroll(current)),
                        ..WidgetRouteEffect::default()
                    });
                }

                if next == current {
                    return Some(WidgetRouteEffect::default());
                }

                *props.state.position_mut() = next;
                Some(WidgetRouteEffect {
                    dirty: true,
                    message: props
                        .on_scroll
                        .as_mut()
                        .and_then(|on_scroll| on_scroll(next)),
                })
            }
            Self::Input(props) => match key {
                WidgetKey::Char(ch) => {
                    insert_char_at_cursor(&mut props.value, props.cursor, ch);
                    props.cursor = props.cursor.saturating_add(1);
                    if let Some(on_change) = props.on_change.as_mut() {
                        on_change(props.value.clone());
                    }

                    Some(WidgetRouteEffect {
                        dirty: true,
                        ..WidgetRouteEffect::default()
                    })
                }
                WidgetKey::Backspace => {
                    if props.cursor == 0 {
                        return Some(WidgetRouteEffect::default());
                    }

                    remove_char_before_cursor(&mut props.value, props.cursor);
                    props.cursor = props.cursor.saturating_sub(1);
                    if let Some(on_change) = props.on_change.as_mut() {
                        on_change(props.value.clone());
                    }

                    Some(WidgetRouteEffect {
                        dirty: true,
                        ..WidgetRouteEffect::default()
                    })
                }
                WidgetKey::Left => {
                    let next = props.cursor.saturating_sub(1);
                    if next == props.cursor {
                        Some(WidgetRouteEffect::default())
                    } else {
                        props.cursor = next;
                        Some(WidgetRouteEffect {
                            dirty: true,
                            ..WidgetRouteEffect::default()
                        })
                    }
                }
                WidgetKey::Right => {
                    let max = props.value.chars().count();
                    if props.cursor >= max {
                        Some(WidgetRouteEffect::default())
                    } else {
                        props.cursor += 1;
                        Some(WidgetRouteEffect {
                            dirty: true,
                            ..WidgetRouteEffect::default()
                        })
                    }
                }
                WidgetKey::Enter => Some(WidgetRouteEffect {
                    message: props
                        .on_submit
                        .as_mut()
                        .and_then(|on_submit| on_submit(props.value.clone())),
                    ..WidgetRouteEffect::default()
                }),
                _ => None,
            },
            _ => None,
        }
    }
}

impl<Message> fmt::Debug for ElementKind<Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Box(props) => f.debug_tuple("Box").field(props).finish(),
            Self::Text(props) => f.debug_tuple("Text").field(props).finish(),
            Self::Pane(props) => f.debug_tuple("Pane").field(props).finish(),
            Self::Block(props) => f.debug_tuple("Block").field(props).finish(),
            Self::Paragraph(props) => f.debug_tuple("Paragraph").field(props).finish(),
            Self::RichText(props) => f.debug_tuple("RichText").field(props).finish(),
            Self::List(props) => f.debug_tuple("List").field(props).finish(),
            Self::Tabs(props) => f.debug_tuple("Tabs").field(props).finish(),
            Self::Gauge(props) => f.debug_tuple("Gauge").field(props).finish(),
            Self::Clear(props) => f.debug_tuple("Clear").field(props).finish(),
            Self::LineGauge(props) => f.debug_tuple("LineGauge").field(props).finish(),
            Self::Table(props) => f.debug_tuple("Table").field(props).finish(),
            Self::Sparkline(props) => f.debug_tuple("Sparkline").field(props).finish(),
            Self::BarChart(props) => f.debug_tuple("BarChart").field(props).finish(),
            Self::Chart(props) => f.debug_tuple("Chart").field(props).finish(),
            Self::Canvas(props) => f.debug_tuple("Canvas").field(props).finish(),
            Self::Monthly(props) => f.debug_tuple("Monthly").field(props).finish(),
            Self::ScrollView(props) => f.debug_tuple("ScrollView").field(props).finish(),
            Self::Scrollbar(props) => f.debug_tuple("Scrollbar").field(props).finish(),
            Self::StreamingText(props) => f.debug_tuple("StreamingText").field(props).finish(),
            Self::Input(props) => f.debug_tuple("Input").field(props).finish(),
            Self::StatusBar(props) => f.debug_tuple("StatusBar").field(props).finish(),
            Self::Shell(props) => f.debug_tuple("Shell").field(props).finish(),
            Self::Component(props) => f.debug_tuple("Component").field(props).finish(),
        }
    }
}

pub struct Element<Message> {
    pub kind: ElementKind<Message>,
    pub layout: Layout,
    pub style: Style,
    pub focusable: bool,
    pub continuity_key: Option<String>,
    pub children: Vec<Element<Message>>,
}

pub trait IntoElement<Message> {
    fn into_element(self) -> Element<Message>;
}

impl<Message> IntoElement<Message> for Element<Message> {
    fn into_element(self) -> Element<Message> {
        self
    }
}

pub fn component<Message, F>(name: impl Into<String>, renderer: F) -> Element<Message>
where
    F: Fn() -> Element<Message> + 'static,
{
    Element::new(ElementKind::Component(ComponentProps {
        name: name.into(),
        scope: None,
        renderer: ComponentRenderer::Static(Rc::new(renderer)),
    }))
}

pub fn component_with_cx<Message, F>(name: impl Into<String>, renderer: F) -> Element<Message>
where
    F: for<'a> Fn(&mut crate::ViewCtx<'a, Message>) -> Element<Message> + 'static,
{
    Element::new(ElementKind::Component(ComponentProps {
        name: name.into(),
        scope: None,
        renderer: ComponentRenderer::WithCx(Rc::new(renderer)),
    }))
}

impl<Message> Element<Message> {
    pub fn new(kind: ElementKind<Message>) -> Self {
        Self {
            kind,
            layout: Layout::default(),
            style: Style::default(),
            focusable: false,
            continuity_key: None,
            children: Vec::new(),
        }
    }

    pub fn new_text(content: impl Into<String>) -> Self {
        Self::new(ElementKind::Text(TextProps {
            content: content.into(),
        }))
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn with_continuity_key(mut self, key: impl Into<String>) -> Self {
        self.continuity_key = Some(key.into());
        self
    }

    pub fn with_children(mut self, children: Vec<Element<Message>>) -> Self {
        self.children = children;
        self
    }

    pub fn continuity_key(&self) -> Option<&str> {
        self.continuity_key.as_deref()
    }

    pub fn child_layout_spec(&self, bounds: Rect) -> ChildLayoutSpec {
        self.kind.child_layout_spec(bounds)
    }

    pub fn intrinsic_height(&self, width: u16, child_heights: &[u16]) -> u16 {
        self.kind.intrinsic_height(width, child_heights)
    }

    pub fn intrinsic_width(&self, child_widths: &[u16]) -> u16 {
        self.kind.intrinsic_width(child_widths)
    }

    pub fn invalidates_self_on_layout_change(&self) -> bool {
        self.kind.invalidates_self_on_layout_change(self.style)
    }

    pub fn kind_name(&self) -> &'static str {
        match &self.kind {
            ElementKind::Box(_) => "Box",
            ElementKind::Text(_) => "Text",
            ElementKind::Pane(_) => "Pane",
            ElementKind::Block(_) => "Block",
            ElementKind::Paragraph(_) => "Paragraph",
            ElementKind::RichText(_) => "RichText",
            ElementKind::List(_) => "List",
            ElementKind::Tabs(_) => "Tabs",
            ElementKind::Gauge(_) => "Gauge",
            ElementKind::Clear(_) => "Clear",
            ElementKind::LineGauge(_) => "LineGauge",
            ElementKind::Table(_) => "Table",
            ElementKind::Sparkline(_) => "Sparkline",
            ElementKind::BarChart(_) => "BarChart",
            ElementKind::Chart(_) => "Chart",
            ElementKind::Canvas(_) => "Canvas",
            ElementKind::Monthly(_) => "Monthly",
            ElementKind::ScrollView(_) => "ScrollView",
            ElementKind::Scrollbar(_) => "Scrollbar",
            ElementKind::StreamingText(_) => "StreamingText",
            ElementKind::Input(_) => "Input",
            ElementKind::StatusBar(_) => "StatusBar",
            ElementKind::Shell(_) => "Shell",
            ElementKind::Component(_) => "Component",
        }
    }
}

impl<Message> fmt::Debug for Element<Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Element")
            .field("kind", &self.kind)
            .field("layout", &self.layout)
            .field("style", &self.style)
            .field("focusable", &self.focusable)
            .field("children_len", &self.children.len())
            .finish()
    }
}

#[derive(Debug)]
pub struct Node<Message> {
    pub id: usize,
    pub rect: Rect,
    pub measured_height: u16,
    pub element: Element<Message>,
    pub children: Vec<Node<Message>>,
}

impl<Message> Node<Message> {
    pub fn child_layout_spec(&self, bounds: Rect) -> ChildLayoutSpec {
        self.element.child_layout_spec(bounds)
    }
}

fn wrapped_line_count(content: &str, width: u16) -> u16 {
    if width == 0 {
        return 0;
    }

    let mut count = 0u16;

    for raw_line in content.split('\n') {
        if raw_line.is_empty() {
            count = count.saturating_add(1);
            continue;
        }

        let mut current_width = 0u16;
        let mut line_count = 1u16;

        for ch in raw_line.chars() {
            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
            let char_width = char_width.max(1);
            if current_width.saturating_add(char_width) > width && current_width > 0 {
                line_count = line_count.saturating_add(1);
                current_width = 0;
            }

            current_width = current_width.saturating_add(char_width);
        }

        count = count.saturating_add(line_count);
    }

    count.max(1)
}

fn text_wrapped_height(text: &Text, width: u16, wrap: bool) -> u16 {
    if width == 0 {
        return 0;
    }

    if !wrap {
        return text.height() as u16;
    }

    text.lines
        .iter()
        .map(|line| wrapped_line_count(&line.plain(), width))
        .fold(0u16, u16::saturating_add)
        .max(1)
}

fn paragraph_height(props: &ParagraphProps, width: u16) -> u16 {
    let (content_width, chrome_height) = if let Some(block) = &props.block {
        let inner_width = width.saturating_sub(border_horizontal_inset(block.props.borders));
        let content_width = inner_width
            .saturating_sub(block.props.padding.left)
            .saturating_sub(block.props.padding.right);
        let chrome_height = block
            .props
            .padding
            .top
            .saturating_add(block.props.padding.bottom)
            .saturating_add(block_vertical_inset(&block.props));
        (content_width, chrome_height)
    } else {
        (width, 0)
    };

    let text_height = text_wrapped_height(&props.content, content_width, props.wrap.is_some());

    text_height.saturating_add(chrome_height).max(1)
}

fn paragraph_width(props: &ParagraphProps) -> u16 {
    let text_width = props.content.width() as u16;
    if let Some(block) = &props.block {
        text_width
            .saturating_add(block.props.padding.left)
            .saturating_add(block.props.padding.right)
            .saturating_add(border_horizontal_inset(block.props.borders))
            .max(block_title_width(&block.props))
    } else {
        text_width
    }
}

fn plain_text_width(content: &str) -> u16 {
    content
        .split('\n')
        .map(|line| {
            line.chars()
                .map(|ch| UnicodeWidthChar::width(ch).unwrap_or(0) as u16)
                .sum::<u16>()
        })
        .max()
        .unwrap_or(0)
}

fn rich_text_width(block: &HistoryBlock) -> u16 {
    block
        .lines
        .iter()
        .map(|line| {
            line.runs
                .iter()
                .map(|run| plain_text_width(&run.text))
                .fold(0u16, u16::saturating_add)
        })
        .max()
        .unwrap_or(0)
}

fn list_height<Message>(props: &ListProps<Message>) -> u16 {
    let content_height = props
        .items
        .iter()
        .map(ListItem::height)
        .sum::<usize>()
        .max(1) as u16;
    block_height(props.block.as_ref(), content_height)
}

fn list_width<Message>(props: &ListProps<Message>) -> u16 {
    let item_width = props.items.iter().map(ListItem::width).max().unwrap_or(0) as u16;
    let highlight_width = props.highlight_symbol.as_ref().map_or(0, |symbol| {
        if props
            .highlight_spacing
            .should_add(props.state.selected().is_some())
        {
            symbol.width() as u16
        } else {
            0
        }
    });
    block_width(
        props.block.as_ref(),
        item_width.saturating_add(highlight_width),
    )
}

fn tabs_height<Message>(props: &TabsProps<Message>) -> u16 {
    block_height(props.block.as_ref(), 1)
}

fn tabs_width<Message>(props: &TabsProps<Message>) -> u16 {
    let titles = props
        .titles
        .iter()
        .map(|title| {
            props.padding_left.width() as u16
                + title.width() as u16
                + props.padding_right.width() as u16
        })
        .fold(0u16, u16::saturating_add);
    let dividers =
        (props.titles.len().saturating_sub(1) as u16).saturating_mul(props.divider.width() as u16);
    block_width(props.block.as_ref(), titles.saturating_add(dividers).max(1))
}

fn block_height(block: Option<&BlockFrame>, content_height: u16) -> u16 {
    let Some(block) = block else {
        return content_height.max(1);
    };

    content_height
        .saturating_add(block.props.padding.top)
        .saturating_add(block.props.padding.bottom)
        .saturating_add(block_vertical_inset(&block.props))
        .max(1)
}

fn block_width(block: Option<&BlockFrame>, content_width: u16) -> u16 {
    let Some(block) = block else {
        return content_width.max(1);
    };

    content_width
        .max(block_title_width(&block.props))
        .saturating_add(block.props.padding.left)
        .saturating_add(block.props.padding.right)
        .saturating_add(border_horizontal_inset(block.props.borders))
        .max(1)
}

fn table_height<Message>(props: &TableProps<Message>) -> u16 {
    let header_rows = props
        .header
        .as_ref()
        .map(Row::height_with_margin)
        .unwrap_or(0);
    let footer_rows = props
        .footer
        .as_ref()
        .map(Row::height_with_margin)
        .unwrap_or(0);
    let body_rows = props
        .rows
        .iter()
        .map(Row::height_with_margin)
        .fold(0u16, u16::saturating_add);
    let content = header_rows
        .saturating_add(body_rows)
        .saturating_add(footer_rows)
        .max(1);

    if let Some(block) = &props.block {
        content
            .saturating_add(block_vertical_inset(&block.props))
            .saturating_add(block.props.padding.top)
            .saturating_add(block.props.padding.bottom)
    } else {
        content
    }
}

fn table_width<Message>(props: &TableProps<Message>) -> u16 {
    let row_width = |row: &Row| -> u16 {
        let cells = row.cells_ref();
        let spacing = cells.len().saturating_sub(1) as u16 * props.column_spacing;
        let content = cells
            .iter()
            .map(|cell| cell.width() as u16)
            .fold(0u16, u16::saturating_add);
        content.saturating_add(spacing)
    };

    let mut content_width = props.header.as_ref().map_or(0, row_width);
    content_width = content_width.max(props.footer.as_ref().map_or(0, row_width));
    content_width = content_width.max(props.rows.iter().map(row_width).max().unwrap_or(0));

    let highlight_width = props.highlight_symbol.as_ref().map_or(0, |symbol| {
        if props
            .highlight_spacing
            .should_add(props.state.selected().is_some())
        {
            symbol.width() as u16
        } else {
            0
        }
    });

    block_width(
        props.block.as_ref(),
        content_width.saturating_add(highlight_width).max(1),
    )
}

fn block_title_width(props: &BlockProps) -> u16 {
    props
        .titles
        .iter()
        .map(|title| title.content.width() as u16)
        .max()
        .unwrap_or(0)
}

fn title_width(title: Option<&str>) -> u16 {
    title.map(plain_text_width).unwrap_or(0)
}

fn border_horizontal_inset(borders: Borders) -> u16 {
    u16::from(borders.contains(Borders::LEFT)) + u16::from(borders.contains(Borders::RIGHT))
}

fn block_vertical_inset(props: &BlockProps) -> u16 {
    let top = u16::from(
        props.borders.contains(Borders::TOP) || props.has_title_at_position(TitlePosition::Top),
    );
    let bottom = u16::from(
        props.borders.contains(Borders::BOTTOM)
            || props.has_title_at_position(TitlePosition::Bottom),
    );
    top.saturating_add(bottom)
}

fn step_list_selection(state: &mut ListState, len: usize, delta: isize) -> Option<usize> {
    if len == 0 {
        return None;
    }

    let mut next = state.selected().unwrap_or(0);
    if delta.is_negative() {
        next = next.saturating_sub(delta.unsigned_abs());
    } else {
        next = next
            .saturating_add(delta as usize)
            .min(len.saturating_sub(1));
    }

    if Some(next) == state.selected() {
        None
    } else {
        state.select(Some(next));
        Some(next)
    }
}

fn sync_list_offset<Message>(props: &mut ListProps<Message>, viewport_height: usize) {
    let Some(selected) = props.state.selected() else {
        return;
    };

    if viewport_height == 0 {
        props.state = props.state.with_offset(0);
        return;
    }

    let item_heights = props.items.iter().map(ListItem::height).collect::<Vec<_>>();
    let selected_top = item_heights[..selected].iter().sum::<usize>();
    let selected_height = item_heights.get(selected).copied().unwrap_or(1).max(1);
    let selected_bottom = selected_top.saturating_add(selected_height);
    let mut offset = props.state.offset();
    let padding = props.scroll_padding;

    let upper_bound = offset.saturating_add(padding);
    if selected_top < upper_bound {
        offset = selected_top.saturating_sub(padding);
    } else {
        let viewport_end = offset.saturating_add(viewport_height);
        let lower_bound = viewport_end.saturating_sub(padding);
        if selected_bottom > lower_bound {
            offset = selected_bottom
                .saturating_add(padding)
                .saturating_sub(viewport_height);
        }
    }

    props.state = props.state.with_offset(offset);
}

fn step_table_selection(state: &mut TableState, len: usize, delta: isize) -> Option<usize> {
    if len == 0 {
        return None;
    }

    let mut next = state.selected().unwrap_or(0);
    if delta.is_negative() {
        next = next.saturating_sub(delta.unsigned_abs());
    } else {
        next = next
            .saturating_add(delta as usize)
            .min(len.saturating_sub(1));
    }

    if Some(next) == state.selected() {
        None
    } else {
        state.select(Some(next));
        Some(next)
    }
}

fn sync_table_offset<Message>(props: &mut TableProps<Message>, viewport_height: usize) {
    let Some(selected) = props.state.selected() else {
        return;
    };

    if viewport_height == 0 {
        props.state = props.state.with_offset(0);
        return;
    }

    let header_rows = props
        .header
        .as_ref()
        .map(Row::height_with_margin)
        .unwrap_or(0) as usize;
    let visible_height = viewport_height.saturating_sub(header_rows).max(1);
    let row_heights = props
        .rows
        .iter()
        .map(Row::height_with_margin)
        .map(usize::from)
        .collect::<Vec<_>>();
    let selected_top = row_heights[..selected].iter().sum::<usize>();
    let selected_height = row_heights.get(selected).copied().unwrap_or(1).max(1);
    let selected_bottom = selected_top.saturating_add(selected_height);
    let mut offset = props.state.offset();

    if selected_top < offset {
        offset = selected_top;
    } else if selected_bottom > offset.saturating_add(visible_height) {
        offset = selected_bottom.saturating_sub(visible_height);
    }

    props.state = props.state.with_offset(offset);
}

fn step_selected_index(selected: &mut Option<usize>, len: usize, delta: isize) -> Option<usize> {
    if len == 0 {
        return None;
    }

    let mut next = selected.unwrap_or(0);
    if delta.is_negative() {
        next = next.saturating_sub(delta.unsigned_abs());
    } else {
        next = next
            .saturating_add(delta as usize)
            .min(len.saturating_sub(1));
    }

    if Some(next) == *selected {
        None
    } else {
        *selected = Some(next);
        Some(next)
    }
}

fn insert_char_at_cursor(value: &mut String, cursor: usize, ch: char) {
    let byte_index = byte_index_for_char(value, cursor);
    value.insert(byte_index, ch);
}

fn remove_char_before_cursor(value: &mut String, cursor: usize) {
    let end = byte_index_for_char(value, cursor);
    let start = byte_index_for_char(value, cursor.saturating_sub(1));
    value.replace_range(start..end, "");
}

fn byte_index_for_char(value: &str, cursor: usize) -> usize {
    if cursor == 0 {
        return 0;
    }

    value
        .char_indices()
        .map(|(index, _)| index)
        .nth(cursor)
        .unwrap_or(value.len())
}

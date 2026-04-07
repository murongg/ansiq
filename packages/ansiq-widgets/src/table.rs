use ansiq_core::{
    BlockFrame, Constraint, Element, ElementKind, Flex, HighlightSpacing, Layout, Length, Row,
    SelectHandler, Style, TableAlignment, TableProps, TableState, Text,
};

pub struct Table<Message = ()> {
    block: Option<BlockFrame>,
    header: Option<Row>,
    footer: Option<Row>,
    rows: Vec<Row>,
    widths: Vec<Constraint>,
    column_spacing: u16,
    flex: Flex,
    alignments: Vec<TableAlignment>,
    state: TableState,
    highlight_symbol: Option<Text>,
    row_highlight_style: Style,
    column_highlight_style: Style,
    cell_highlight_style: Style,
    highlight_spacing: HighlightSpacing,
    on_select: Option<SelectHandler<Message>>,
    layout: Layout,
    style: Style,
    focusable: bool,
}

impl<Message> Default for Table<Message> {
    fn default() -> Self {
        Self {
            block: None,
            header: None,
            footer: None,
            rows: Vec::new(),
            widths: Vec::new(),
            column_spacing: 1,
            flex: Flex::Start,
            alignments: Vec::new(),
            state: TableState::default(),
            highlight_symbol: None,
            row_highlight_style: Style::default(),
            column_highlight_style: Style::default(),
            cell_highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            on_select: None,
            layout: Layout {
                width: Length::Fill,
                height: Length::Auto,
            },
            style: Style::default(),
            focusable: false,
        }
    }
}

impl Table<()> {
    pub fn new<R, T, C>(rows: R, widths: C) -> Self
    where
        R: IntoIterator<Item = T>,
        T: Into<Row>,
        C: IntoIterator,
        C::Item: Into<Constraint>,
    {
        Self::default().rows(rows).widths(widths)
    }
}

impl<Message> Table<Message> {
    pub fn rows<I, T>(mut self, rows: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Row>,
    {
        self.rows = rows.into_iter().map(Into::into).collect();
        self
    }

    pub fn row<T>(mut self, row: T) -> Self
    where
        T: Into<Row>,
    {
        self.rows.push(row.into());
        self
    }

    pub fn header<T>(mut self, row: T) -> Self
    where
        T: Into<Row>,
    {
        self.header = Some(row.into());
        self
    }

    pub fn headers<T>(self, row: T) -> Self
    where
        T: Into<Row>,
    {
        self.header(row)
    }

    pub fn footer<T>(mut self, row: T) -> Self
    where
        T: Into<Row>,
    {
        self.footer = Some(row.into());
        self
    }

    pub fn widths<I>(mut self, widths: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        self.widths = widths.into_iter().map(Into::into).collect();
        self
    }

    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self
    }

    pub fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
        self
    }

    pub fn block(mut self, block: crate::Block<Message>) -> Self {
        self.block = Some(block.into_frame());
        self
    }

    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.state.select(selected);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        *self.state.offset_mut() = offset;
        self
    }

    pub fn state(mut self, state: TableState) -> Self {
        self.state = state;
        self
    }

    pub fn highlight_symbol<T>(mut self, symbol: T) -> Self
    where
        T: Into<Text>,
    {
        self.highlight_symbol = Some(symbol.into());
        self
    }

    pub fn highlight_style<S: Into<Style>>(self, style: S) -> Self {
        self.row_highlight_style(style)
    }

    pub fn row_highlight_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.row_highlight_style = style.into();
        self
    }

    pub fn column_highlight_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.column_highlight_style = style.into();
        self
    }

    pub fn cell_highlight_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.cell_highlight_style = style.into();
        self
    }

    pub fn highlight_spacing(mut self, spacing: HighlightSpacing) -> Self {
        self.highlight_spacing = spacing;
        self
    }

    pub fn alignment(mut self, alignment: TableAlignment) -> Self {
        self.alignments.push(alignment);
        self
    }

    pub fn alignments<I>(mut self, alignments: I) -> Self
    where
        I: IntoIterator<Item = TableAlignment>,
    {
        self.alignments.extend(alignments);
        self
    }

    pub fn on_select<NextMessage, F>(self, handler: F) -> Table<NextMessage>
    where
        F: FnMut(usize) -> Option<NextMessage> + 'static,
    {
        Table {
            block: self.block,
            header: self.header,
            footer: self.footer,
            rows: self.rows,
            widths: self.widths,
            column_spacing: self.column_spacing,
            flex: self.flex,
            alignments: self.alignments,
            state: self.state,
            highlight_symbol: self.highlight_symbol,
            row_highlight_style: self.row_highlight_style,
            column_highlight_style: self.column_highlight_style,
            cell_highlight_style: self.cell_highlight_style,
            highlight_spacing: self.highlight_spacing,
            on_select: Some(std::boxed::Box::new(handler)),
            layout: self.layout,
            style: self.style,
            focusable: true,
        }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    pub fn build(self) -> Element<Message> {
        Element::new(ElementKind::Table(TableProps {
            block: self.block,
            header: self.header,
            footer: self.footer,
            rows: self.rows,
            widths: self.widths,
            column_spacing: self.column_spacing,
            flex: self.flex,
            alignments: self.alignments,
            state: self.state,
            highlight_symbol: self.highlight_symbol,
            row_highlight_style: self.row_highlight_style,
            column_highlight_style: self.column_highlight_style,
            cell_highlight_style: self.cell_highlight_style,
            highlight_spacing: self.highlight_spacing,
            on_select: self.on_select,
        }))
        .with_layout(self.layout)
        .with_style(self.style)
        .with_focusable(self.focusable)
    }
}

impl<Item> FromIterator<Item> for Table<()>
where
    Item: Into<Row>,
{
    fn from_iter<Iter: IntoIterator<Item = Item>>(iter: Iter) -> Self {
        Self::default().rows(iter)
    }
}

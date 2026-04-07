use crate::{Style, Text};

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Cell {
    content: Text,
    style: Style,
    column_span: u16,
}

impl Cell {
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Text>,
    {
        Self {
            content: content.into(),
            style: Style::default(),
            column_span: 1,
        }
    }

    pub fn content<T>(mut self, content: T) -> Self
    where
        T: Into<Text>,
    {
        self.content = content.into();
        self
    }

    pub const fn column_span(mut self, column_span: u16) -> Self {
        self.column_span = column_span;
        self
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    pub fn text(&self) -> &Text {
        &self.content
    }

    pub const fn style_value(&self) -> Style {
        self.style
    }

    pub const fn column_span_value(&self) -> u16 {
        self.column_span
    }

    pub fn width(&self) -> usize {
        self.content.width()
    }

    pub fn height(&self) -> usize {
        self.content.height()
    }
}

impl<T> From<T> for Cell
where
    T: Into<Text>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Row {
    cells: Vec<Cell>,
    height: u16,
    top_margin: u16,
    bottom_margin: u16,
    style: Style,
}

impl Row {
    pub fn new<T>(cells: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<Cell>,
    {
        let cells: Vec<Cell> = cells.into_iter().map(Into::into).collect();
        let height = cells
            .iter()
            .map(|cell| cell.height() as u16)
            .max()
            .unwrap_or(1)
            .max(1);
        Self {
            cells,
            height,
            top_margin: 0,
            bottom_margin: 0,
            style: Style::default(),
        }
    }

    pub fn cells<T>(mut self, cells: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<Cell>,
    {
        self.cells = cells.into_iter().map(Into::into).collect();
        self.height = self
            .cells
            .iter()
            .map(|cell| cell.height() as u16)
            .max()
            .unwrap_or(1)
            .max(1);
        self
    }

    pub const fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    pub const fn top_margin(mut self, margin: u16) -> Self {
        self.top_margin = margin;
        self
    }

    pub const fn bottom_margin(mut self, margin: u16) -> Self {
        self.bottom_margin = margin;
        self
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    pub fn cells_ref(&self) -> &[Cell] {
        &self.cells
    }

    pub const fn height_value(&self) -> u16 {
        self.height
    }

    pub const fn top_margin_value(&self) -> u16 {
        self.top_margin
    }

    pub const fn bottom_margin_value(&self) -> u16 {
        self.bottom_margin
    }

    pub const fn style_value(&self) -> Style {
        self.style
    }

    pub const fn height_with_margin(&self) -> u16 {
        self.height
            .saturating_add(self.top_margin)
            .saturating_add(self.bottom_margin)
    }

    pub fn column_count(&self) -> usize {
        self.cells
            .iter()
            .map(|cell| cell.column_span.max(1) as usize)
            .sum()
    }
}

impl<T> From<T> for Row
where
    T: IntoIterator,
    T::Item: Into<Cell>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<Item> FromIterator<Item> for Row
where
    Item: Into<Cell>,
{
    fn from_iter<Iter: IntoIterator<Item = Item>>(iter: Iter) -> Self {
        Self::new(iter)
    }
}

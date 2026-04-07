use ansiq_core::{
    BlockFrame, Element, ElementKind, HighlightSpacing, Layout, Length, Line, ListDirection,
    ListItem, ListProps, ListState, SelectHandler, Style,
};

use crate::Block;

pub struct List<Message = ()> {
    block: Option<BlockFrame>,
    items: Vec<ListItem>,
    state: ListState,
    highlight_symbol: Option<Line>,
    highlight_style: Style,
    highlight_spacing: HighlightSpacing,
    repeat_highlight_symbol: bool,
    direction: ListDirection,
    scroll_padding: usize,
    on_select: Option<SelectHandler<Message>>,
    layout: Layout,
    style: Style,
    focusable: bool,
}

impl<Message> Default for List<Message> {
    fn default() -> Self {
        Self {
            block: None,
            items: Vec::new(),
            state: ListState::default(),
            highlight_symbol: None,
            highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            repeat_highlight_symbol: false,
            direction: ListDirection::TopToBottom,
            scroll_padding: 0,
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

impl List<()> {
    pub fn new<I, T>(items: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<ListItem>,
    {
        Self::default().items(items)
    }
}

impl<Message> List<Message> {
    pub fn block(mut self, block: Block<Message>) -> Self {
        self.block = Some(block.into_frame());
        self
    }

    pub fn item<T>(mut self, item: T) -> Self
    where
        T: Into<ListItem>,
    {
        self.items.push(item.into());
        self
    }

    pub fn items<I, T>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<ListItem>,
    {
        self.items = items.into_iter().map(Into::into).collect();
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

    pub fn state(mut self, state: ListState) -> Self {
        self.state = state;
        self
    }

    pub fn highlight_symbol<L: Into<Line>>(mut self, symbol: L) -> Self {
        self.highlight_symbol = Some(symbol.into());
        self
    }

    pub fn highlight_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.highlight_style = style.into();
        self
    }

    pub fn highlight_spacing(mut self, spacing: HighlightSpacing) -> Self {
        self.highlight_spacing = spacing;
        self
    }

    pub fn repeat_highlight_symbol(mut self, repeat: bool) -> Self {
        self.repeat_highlight_symbol = repeat;
        self
    }

    pub fn direction(mut self, direction: ListDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn scroll_padding(mut self, padding: usize) -> Self {
        self.scroll_padding = padding;
        self
    }

    pub fn on_select<NextMessage, F>(self, handler: F) -> List<NextMessage>
    where
        F: FnMut(usize) -> Option<NextMessage> + 'static,
    {
        List {
            block: self.block,
            items: self.items,
            state: self.state,
            highlight_symbol: self.highlight_symbol,
            highlight_style: self.highlight_style,
            highlight_spacing: self.highlight_spacing,
            repeat_highlight_symbol: self.repeat_highlight_symbol,
            direction: self.direction,
            scroll_padding: self.scroll_padding,
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

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn build(self) -> Element<Message> {
        Element::new(ElementKind::List(ListProps {
            block: self.block,
            items: self.items,
            state: self.state,
            highlight_symbol: self.highlight_symbol,
            highlight_style: self.highlight_style,
            highlight_spacing: self.highlight_spacing,
            repeat_highlight_symbol: self.repeat_highlight_symbol,
            direction: self.direction,
            scroll_padding: self.scroll_padding,
            on_select: self.on_select,
        }))
        .with_layout(self.layout)
        .with_style(self.style)
        .with_focusable(self.focusable)
    }
}

impl<Item> FromIterator<Item> for List<()>
where
    Item: Into<ListItem>,
{
    fn from_iter<Iter: IntoIterator<Item = Item>>(iter: Iter) -> Self {
        Self::new(iter)
    }
}

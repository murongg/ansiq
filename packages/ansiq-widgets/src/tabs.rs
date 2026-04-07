use ansiq_core::{
    BlockFrame, Element, ElementKind, Layout, Length, Line, SelectHandler, Span, Style, TabsProps,
};

use crate::Block;

pub struct Tabs<Message = ()> {
    block: Option<BlockFrame>,
    titles: Vec<Line>,
    selected: Option<usize>,
    selection_explicit: bool,
    highlight_style: Style,
    divider: Span,
    padding_left: Line,
    padding_right: Line,
    on_select: Option<SelectHandler<Message>>,
    layout: Layout,
    style: Style,
    focusable: bool,
}

impl<Message> Default for Tabs<Message> {
    fn default() -> Self {
        Self {
            block: None,
            titles: Vec::new(),
            selected: None,
            selection_explicit: false,
            highlight_style: Style::default().reversed(true),
            divider: Span::raw(ansiq_core::symbols::line::VERTICAL),
            padding_left: Line::from(" "),
            padding_right: Line::from(" "),
            on_select: None,
            layout: Layout {
                width: Length::Fill,
                height: Length::Fixed(1),
            },
            style: Style::default(),
            focusable: false,
        }
    }
}

impl Tabs<()> {
    pub fn new<I, T>(titles: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Line>,
    {
        Self::default().titles(titles)
    }
}

impl<Message> Tabs<Message> {
    pub fn block(mut self, block: Block<Message>) -> Self {
        self.block = Some(block.into_frame());
        self
    }

    pub fn tab<T>(mut self, title: T) -> Self
    where
        T: Into<Line>,
    {
        self.titles.push(title.into());
        if self.titles.len() == 1 && self.selected.is_none() {
            self.selected = Some(0);
        }
        self
    }

    pub fn titles<I, T>(mut self, titles: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Line>,
    {
        self.titles = titles.into_iter().map(Into::into).collect();
        self.selected = if self.titles.is_empty() {
            None
        } else {
            self.selected
                .map(|selected| selected.min(self.titles.len() - 1))
                .or(Some(0))
        };
        self
    }

    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self.selection_explicit = true;
        self
    }

    pub fn select<T>(mut self, selected: T) -> Self
    where
        T: Into<Option<usize>>,
    {
        self.selected = selected.into();
        self.selection_explicit = true;
        self
    }

    pub fn highlight_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.highlight_style = style.into();
        self
    }

    pub fn divider<D: Into<Span>>(mut self, divider: D) -> Self {
        self.divider = divider.into();
        self
    }

    pub fn padding<L, R>(mut self, left: L, right: R) -> Self
    where
        L: Into<Line>,
        R: Into<Line>,
    {
        self.padding_left = left.into();
        self.padding_right = right.into();
        self
    }

    pub fn padding_left<P: Into<Line>>(mut self, left: P) -> Self {
        self.padding_left = left.into();
        self
    }

    pub fn padding_right<P: Into<Line>>(mut self, right: P) -> Self {
        self.padding_right = right.into();
        self
    }

    pub fn on_select<NextMessage, F>(self, handler: F) -> Tabs<NextMessage>
    where
        F: FnMut(usize) -> Option<NextMessage> + 'static,
    {
        Tabs {
            block: self.block,
            titles: self.titles,
            selected: self.selected,
            selection_explicit: self.selection_explicit,
            highlight_style: self.highlight_style,
            divider: self.divider,
            padding_left: self.padding_left,
            padding_right: self.padding_right,
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
        Element::new(ElementKind::Tabs(TabsProps {
            block: self.block,
            titles: self.titles,
            selected: self.selected,
            selection_explicit: self.selection_explicit,
            highlight_style: self.highlight_style,
            divider: self.divider,
            padding_left: self.padding_left,
            padding_right: self.padding_right,
            on_select: self.on_select,
        }))
        .with_layout(self.layout)
        .with_style(self.style)
        .with_focusable(self.focusable)
    }
}

impl<Item> FromIterator<Item> for Tabs<()>
where
    Item: Into<Line>,
{
    fn from_iter<Iter: IntoIterator<Item = Item>>(iter: Iter) -> Self {
        Self::new(iter)
    }
}

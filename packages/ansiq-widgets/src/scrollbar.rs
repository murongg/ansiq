use ansiq_core::{
    Element, ElementKind, Layout, Length, ScrollHandler, ScrollbarOrientation, ScrollbarProps,
    ScrollbarState, Style, symbols::scrollbar::Set as ScrollbarSymbolSet,
};

fn default_symbols(orientation: ScrollbarOrientation) -> ScrollbarSymbolSet {
    if orientation.is_vertical() {
        ansiq_core::symbols::scrollbar::DOUBLE_VERTICAL
    } else {
        ansiq_core::symbols::scrollbar::DOUBLE_HORIZONTAL
    }
}

pub struct Scrollbar<Message = ()> {
    orientation: ScrollbarOrientation,
    state: ScrollbarState,
    thumb_symbol: String,
    thumb_style: Style,
    track_symbol: Option<String>,
    track_style: Style,
    begin_symbol: Option<String>,
    begin_style: Style,
    end_symbol: Option<String>,
    end_style: Style,
    on_scroll: Option<ScrollHandler<Message>>,
    layout: Layout,
    style: Style,
    focusable: bool,
}

impl<Message> Default for Scrollbar<Message> {
    fn default() -> Self {
        Self::new(ScrollbarOrientation::VerticalRight)
    }
}

impl<Message> Scrollbar<Message> {
    pub fn new(orientation: ScrollbarOrientation) -> Self {
        let symbols = default_symbols(orientation);
        Self {
            orientation,
            state: ScrollbarState::default(),
            thumb_symbol: symbols.thumb.to_string(),
            thumb_style: Style::default(),
            track_symbol: Some(symbols.track.to_string()),
            track_style: Style::default(),
            begin_symbol: Some(symbols.begin.to_string()),
            begin_style: Style::default(),
            end_symbol: Some(symbols.end.to_string()),
            end_style: Style::default(),
            on_scroll: None,
            layout: match orientation {
                ScrollbarOrientation::VerticalRight | ScrollbarOrientation::VerticalLeft => {
                    Layout {
                        width: Length::Fixed(1),
                        height: Length::Fill,
                    }
                }
                ScrollbarOrientation::HorizontalBottom | ScrollbarOrientation::HorizontalTop => {
                    Layout {
                        width: Length::Fill,
                        height: Length::Fixed(1),
                    }
                }
            },
            style: Style::default(),
            focusable: false,
        }
    }

    pub fn state(mut self, state: ScrollbarState) -> Self {
        self.state = state;
        self
    }

    pub fn position(mut self, position: usize) -> Self {
        self.state = self.state.position(position);
        self
    }

    pub fn content_length(mut self, content_length: usize) -> Self {
        self.state = self.state.content_length(content_length);
        self
    }

    pub fn viewport_length(self, viewport_length: usize) -> Self {
        self.viewport_content_length(viewport_length)
    }

    pub fn viewport_content_length(mut self, viewport_length: usize) -> Self {
        self.state = self.state.viewport_content_length(viewport_length);
        self
    }

    pub fn orientation(mut self, orientation: ScrollbarOrientation) -> Self {
        self.orientation = orientation;
        let symbols = default_symbols(orientation);
        self = self.symbols(symbols);
        self.layout = match orientation {
            ScrollbarOrientation::VerticalRight | ScrollbarOrientation::VerticalLeft => Layout {
                width: Length::Fixed(1),
                height: Length::Fill,
            },
            ScrollbarOrientation::HorizontalBottom | ScrollbarOrientation::HorizontalTop => {
                Layout {
                    width: Length::Fill,
                    height: Length::Fixed(1),
                }
            }
        };
        self
    }

    pub fn orientation_and_symbol(
        mut self,
        orientation: ScrollbarOrientation,
        symbols: ScrollbarSymbolSet,
    ) -> Self {
        self.orientation = orientation;
        self = self.symbols(symbols);
        self.layout = match orientation {
            ScrollbarOrientation::VerticalRight | ScrollbarOrientation::VerticalLeft => Layout {
                width: Length::Fixed(1),
                height: Length::Fill,
            },
            ScrollbarOrientation::HorizontalBottom | ScrollbarOrientation::HorizontalTop => {
                Layout {
                    width: Length::Fill,
                    height: Length::Fixed(1),
                }
            }
        };
        self
    }

    pub fn symbols(mut self, symbols: ScrollbarSymbolSet) -> Self {
        self.thumb_symbol = symbols.thumb.to_string();
        if self.track_symbol.is_some() {
            self.track_symbol = Some(symbols.track.to_string());
        }
        if self.begin_symbol.is_some() {
            self.begin_symbol = Some(symbols.begin.to_string());
        }
        if self.end_symbol.is_some() {
            self.end_symbol = Some(symbols.end.to_string());
        }
        self
    }

    pub fn thumb_symbol(mut self, thumb_symbol: impl Into<String>) -> Self {
        self.thumb_symbol = thumb_symbol.into();
        self
    }

    pub fn thumb_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.thumb_style = style.into();
        self
    }

    pub fn track_symbol<S: Into<String>>(mut self, track_symbol: Option<S>) -> Self {
        self.track_symbol = track_symbol.map(Into::into);
        self
    }

    pub fn track_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.track_style = style.into();
        self
    }

    pub fn begin_symbol<S: Into<String>>(mut self, begin_symbol: Option<S>) -> Self {
        self.begin_symbol = begin_symbol.map(Into::into);
        self
    }

    pub fn begin_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.begin_style = style.into();
        self
    }

    pub fn end_symbol<S: Into<String>>(mut self, end_symbol: Option<S>) -> Self {
        self.end_symbol = end_symbol.map(Into::into);
        self
    }

    pub fn end_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.end_style = style.into();
        self
    }

    pub fn on_scroll<F>(mut self, handler: F) -> Self
    where
        F: FnMut(usize) -> Option<Message> + 'static,
    {
        self.on_scroll = Some(std::boxed::Box::new(handler));
        self.focusable = true;
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        let style = style.into();
        self.style = style;
        self.thumb_style = style;
        self.track_style = style;
        self.begin_style = style;
        self.end_style = style;
        self
    }

    pub fn build(self) -> Element<Message> {
        Element::new(ElementKind::Scrollbar(ScrollbarProps {
            orientation: self.orientation,
            state: self.state,
            thumb_symbol: self.thumb_symbol,
            thumb_style: self.thumb_style,
            track_symbol: self.track_symbol,
            track_style: self.track_style,
            begin_symbol: self.begin_symbol,
            begin_style: self.begin_style,
            end_symbol: self.end_symbol,
            end_style: self.end_style,
            on_scroll: self.on_scroll,
        }))
        .with_layout(self.layout)
        .with_style(self.style)
        .with_focusable(self.focusable)
    }
}

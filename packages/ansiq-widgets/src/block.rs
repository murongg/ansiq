use ansiq_core::{
    Alignment, BlockFrame, BlockProps, BlockTitle, BorderType, Borders, Element, ElementKind,
    Layout, Length, Line, Padding, Rect, Style, TitlePosition,
};

pub struct Block<Message = ()> {
    element: Element<Message>,
}

impl<Message> Block<Message> {
    pub fn new() -> Self {
        Self {
            element: Element::new(ElementKind::Block(BlockProps {
                titles: Vec::new(),
                title_alignment: Alignment::Left,
                title_position: TitlePosition::Top,
                borders: Borders::NONE,
                border_type: BorderType::Plain,
                border_set: None,
                padding: Padding::zero(),
                border_style: Style::default(),
                title_style: Style::default(),
            }))
            .with_layout(Layout {
                width: Length::Fill,
                height: Length::Auto,
            }),
        }
    }

    pub fn bordered() -> Self {
        let mut block = Self::new();
        if let ElementKind::Block(props) = &mut block.element.kind {
            props.borders = Borders::ALL;
        }
        block
    }

    pub fn title<T>(mut self, title: T) -> Self
    where
        T: Into<Line>,
    {
        if let ElementKind::Block(props) = &mut self.element.kind {
            props.titles.push(BlockTitle::new(title));
        }
        self
    }

    pub fn title_top<T>(mut self, title: T) -> Self
    where
        T: Into<Line>,
    {
        if let ElementKind::Block(props) = &mut self.element.kind {
            props.titles.push(BlockTitle::top(title));
        }
        self
    }

    pub fn title_bottom<T>(mut self, title: T) -> Self
    where
        T: Into<Line>,
    {
        if let ElementKind::Block(props) = &mut self.element.kind {
            props.titles.push(BlockTitle::bottom(title));
        }
        self
    }

    pub fn title_alignment(mut self, alignment: Alignment) -> Self {
        if let ElementKind::Block(props) = &mut self.element.kind {
            props.title_alignment = alignment;
        }
        self
    }

    pub fn title_position(mut self, position: TitlePosition) -> Self {
        if let ElementKind::Block(props) = &mut self.element.kind {
            props.title_position = position;
        }
        self
    }

    pub fn bordered_flag(mut self, bordered: bool) -> Self {
        if let ElementKind::Block(props) = &mut self.element.kind {
            props.borders = if bordered {
                Borders::ALL
            } else {
                Borders::NONE
            };
        }
        self
    }

    pub fn borders(mut self, borders: Borders) -> Self {
        if let ElementKind::Block(props) = &mut self.element.kind {
            props.borders = borders;
        }
        self
    }

    pub fn border_type(mut self, border_type: BorderType) -> Self {
        if let ElementKind::Block(props) = &mut self.element.kind {
            props.border_type = border_type;
        }
        self
    }

    pub fn border_set(mut self, border_set: ansiq_core::symbols::border::Set) -> Self {
        if let ElementKind::Block(props) = &mut self.element.kind {
            props.border_set = Some(border_set);
        }
        self
    }

    pub fn padding(mut self, padding: Padding) -> Self {
        if let ElementKind::Block(props) = &mut self.element.kind {
            props.padding = padding;
        }
        self
    }

    pub fn border_style<S: Into<Style>>(mut self, style: S) -> Self {
        if let ElementKind::Block(props) = &mut self.element.kind {
            props.border_style = style.into();
        }
        self
    }

    pub fn title_style<S: Into<Style>>(mut self, style: S) -> Self {
        if let ElementKind::Block(props) = &mut self.element.kind {
            props.title_style = style.into();
        }
        self
    }

    pub fn child(mut self, child: Element<Message>) -> Self {
        self.element.children.push(child);
        self
    }

    pub fn children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = Element<Message>>,
    {
        self.element.children.extend(children);
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.element.layout = layout;
        self
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.element.style = style.into();
        self
    }

    pub fn build(self) -> Element<Message> {
        self.element
    }

    pub fn inner(&self, rect: Rect) -> Rect {
        let ElementKind::Block(props) = &self.element.kind else {
            unreachable!("Block widgets always store Block props")
        };

        props.inner(rect)
    }

    pub fn into_frame(self) -> BlockFrame {
        let style = self.element.style;
        let ElementKind::Block(props) = self.element.kind else {
            unreachable!("Block widgets always store Block props")
        };

        BlockFrame { props, style }
    }
}

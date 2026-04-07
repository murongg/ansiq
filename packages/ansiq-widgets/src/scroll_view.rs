use ansiq_core::{Element, ElementKind, Layout, Length, ScrollHandler, ScrollViewProps};

pub struct ScrollView<Message = ()> {
    child: Option<Element<Message>>,
    follow_bottom: bool,
    offset: Option<usize>,
    on_scroll: Option<ScrollHandler<Message>>,
    layout: Layout,
    focusable: bool,
}

impl<Message> ScrollView<Message> {
    pub fn new() -> Self {
        Self {
            child: None,
            follow_bottom: false,
            offset: None,
            on_scroll: None,
            layout: Layout {
                width: Length::Fill,
                height: Length::Fill,
            },
            focusable: false,
        }
    }
    pub fn follow_bottom(mut self, follow_bottom: bool) -> Self {
        self.follow_bottom = follow_bottom;
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
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

    pub fn child(mut self, child: Element<Message>) -> Self {
        self.child = Some(child);
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn build(self) -> Element<Message> {
        Element::new(ElementKind::ScrollView(ScrollViewProps {
            follow_bottom: self.follow_bottom,
            offset: self.offset,
            on_scroll: self.on_scroll,
        }))
        .with_layout(self.layout)
        .with_focusable(self.focusable)
        .with_children(self.child.into_iter().collect())
    }
}

use ansiq_core::{Element, ElementKind, Layout, Length, ShellProps, Style};

pub struct Shell<Message = ()> {
    element: Element<Message>,
}

impl<Message> Shell<Message> {
    pub fn new() -> Self {
        Self {
            element: Element::new(ElementKind::Shell(ShellProps)).with_layout(Layout {
                width: Length::Fill,
                height: Length::Fill,
            }),
        }
    }

    pub fn header(mut self, child: Element<Message>) -> Self {
        set_shell_slot(&mut self.element.children, 0, child);
        self
    }

    pub fn body(mut self, child: Element<Message>) -> Self {
        set_shell_slot(&mut self.element.children, 1, child);
        self
    }

    pub fn footer(mut self, child: Element<Message>) -> Self {
        set_shell_slot(&mut self.element.children, 2, child);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.element.style = style;
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.element.layout = layout;
        self
    }

    pub fn build(self) -> Element<Message> {
        self.element
    }
}

fn set_shell_slot<Message>(
    children: &mut Vec<Element<Message>>,
    index: usize,
    child: Element<Message>,
) {
    if children.len() <= index {
        children.resize_with(index + 1, || {
            Element::new(ElementKind::Text(ansiq_core::TextProps {
                content: String::new(),
            }))
            .with_layout(Layout {
                width: Length::Fill,
                height: Length::Auto,
            })
        });
    }

    children[index] = child;
}

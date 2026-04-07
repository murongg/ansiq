use ansiq_core::{Element, Layout, Length, ViewCtx};
use ansiq_runtime::{App, Engine, RuntimeHandle};
use ansiq_surface::ViewportPolicy;
use ansiq_widgets::{Block, Box, List, Paragraph, ScrollView, Shell, StatusBar, Text};

use crate::openapi_explorer::{OpenApiDocumentView, OperationView, SchemaView, parse_document};

pub const VIEWPORT_POLICY: ViewportPolicy = ViewportPolicy::ReserveFitContent { min: 22, max: 32 };

#[derive(Clone, Debug)]
pub enum OpenApiExplorerMessage {
    SelectOperation(usize),
}

pub struct OpenApiExplorerApp {
    document: OpenApiDocumentView,
    selected_operation: usize,
}

impl OpenApiExplorerApp {
    pub fn from_spec_text(source: &str, text: &str) -> Result<Self, String> {
        let document = parse_document(source, text)?;
        Ok(Self {
            document,
            selected_operation: 0,
        })
    }

    fn operations(&self) -> Vec<&OperationView> {
        self.document
            .tags
            .iter()
            .flat_map(|tag| tag.operations.iter())
            .collect()
    }

    fn selected_operation(&self) -> Option<&OperationView> {
        self.operations().get(self.selected_operation).copied()
    }

    fn selected_schema(&self) -> Option<&SchemaView> {
        let operation = self.selected_operation()?;
        let first = operation.schema_refs.first()?;
        self.document.schema_index.get(first)
    }

    fn navigation_items(&self) -> Vec<String> {
        self.document
            .tags
            .iter()
            .flat_map(|tag| {
                tag.operations.iter().map(|operation| {
                    format!("[{}] {} {}", tag.name, operation.method, operation.path)
                })
            })
            .collect()
    }

    fn operation_text(&self) -> String {
        let Some(operation) = self.selected_operation() else {
            return "No operation selected".to_string();
        };

        let mut lines = vec![
            format!("{} {}", operation.method, operation.path),
            operation.summary.clone(),
        ];
        if !operation.description.is_empty() {
            lines.push(String::new());
            lines.push(operation.description.clone());
        }
        if !operation.parameters.is_empty() {
            lines.push(String::new());
            lines.push("Parameters".to_string());
            lines.extend(operation.parameters.iter().map(|line| format!("- {line}")));
        }
        if !operation.request_body.is_empty() {
            lines.push(String::new());
            lines.push("Request Body".to_string());
            lines.extend(
                operation
                    .request_body
                    .iter()
                    .map(|line| format!("- {line}")),
            );
        }
        if !operation.responses.is_empty() {
            lines.push(String::new());
            lines.push("Responses".to_string());
            lines.extend(operation.responses.iter().map(|line| format!("- {line}")));
        }
        if !operation.schema_refs.is_empty() {
            lines.push(String::new());
            lines.push("Schema Refs".to_string());
            lines.extend(operation.schema_refs.iter().map(|line| format!("- {line}")));
        }
        lines.join("\n")
    }

    fn schema_text(&self) -> String {
        self.selected_schema()
            .map(|schema| format!("{}\n\n{}", schema.title, schema.lines.join("\n")))
            .unwrap_or_else(|| "No schema selected".to_string())
    }
}

impl App for OpenApiExplorerApp {
    type Message = OpenApiExplorerMessage;

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        let titles = vec![
            "Operations".to_string(),
            "Operation".to_string(),
            "Schema".to_string(),
        ];

        Shell::<OpenApiExplorerMessage>::new()
            .header(
                Box::<OpenApiExplorerMessage>::column()
                    .gap(1)
                    .child(
                        StatusBar::<OpenApiExplorerMessage>::new(format!(
                            "> openapi explorer · {} · {}",
                            self.document.title, self.document.source_label
                        ))
                        .build(),
                    )
                    .child(Text::<OpenApiExplorerMessage>::new(titles.join(" · ")).build())
                    .build(),
            )
            .body(
                Box::row()
                    .gap(1)
                    .child(
                        Block::bordered()
                            .title("Operations")
                            .layout(Layout {
                                width: Length::Fill,
                                height: Length::Fill,
                            })
                            .child(
                                List::new(self.navigation_items())
                                    .selected(Some(self.selected_operation))
                                    .on_select(|index| {
                                        Some(OpenApiExplorerMessage::SelectOperation(index))
                                    })
                                    .layout(Layout {
                                        width: Length::Fill,
                                        height: Length::Fill,
                                    })
                                    .build(),
                            )
                            .build(),
                    )
                    .child(
                        Block::bordered()
                            .title("Operation")
                            .layout(Layout {
                                width: Length::Fill,
                                height: Length::Fill,
                            })
                            .child(
                                ScrollView::new()
                                    .child(
                                        Paragraph::new(self.operation_text())
                                            .layout(Layout {
                                                width: Length::Fill,
                                                height: Length::Auto,
                                            })
                                            .build(),
                                    )
                                    .layout(Layout {
                                        width: Length::Fill,
                                        height: Length::Fill,
                                    })
                                    .build(),
                            )
                            .build(),
                    )
                    .child(
                        Block::bordered()
                            .title("Schema")
                            .layout(Layout {
                                width: Length::Fill,
                                height: Length::Fill,
                            })
                            .child(
                                ScrollView::new()
                                    .child(
                                        Paragraph::new(self.schema_text())
                                            .layout(Layout {
                                                width: Length::Fill,
                                                height: Length::Auto,
                                            })
                                            .build(),
                                    )
                                    .layout(Layout {
                                        width: Length::Fill,
                                        height: Length::Fill,
                                    })
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .build()
    }

    fn update(&mut self, message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {
        match message {
            OpenApiExplorerMessage::SelectOperation(index) => {
                let max = self.operations().len().saturating_sub(1);
                self.selected_operation = index.min(max);
            }
        }
    }
}

pub fn rendered_screen_for_test(
    engine: &Engine<OpenApiExplorerApp>,
    width: u16,
    height: u16,
) -> String {
    let tree = engine.tree().expect("tree should exist");
    let mut buffer = ansiq_render::FrameBuffer::new(width, height);
    ansiq_runtime::draw_tree(tree, engine.focused(), &mut buffer);

    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer.get(x, y).symbol)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

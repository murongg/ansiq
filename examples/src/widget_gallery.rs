use ansiq_core::{
    Constraint, Element, HistoryBlock, HistoryLine, HistoryRun, Layout, Length, Line, Padding, Row,
    ScrollbarOrientation, Style, TranscriptEntry, TranscriptRole, TranscriptSession, Wrap,
};
use ansiq_runtime::{App, RuntimeHandle, ViewportPolicy};
use ansiq_widgets::{
    BarChart, Block, BottomPane, Box, Canvas, Chart, Clear, ComposerBar, Gauge, Input, LineGauge,
    List, Monthly, Pane, Paragraph, RichText, ScrollView, Scrollbar, SessionHeader,
    SessionTranscript, Shell, Sparkline, StatusBar, StreamingText, Table, Tabs, Text,
    TranscriptView,
};

pub const VIEWPORT_POLICY: ViewportPolicy = ViewportPolicy::ReserveFitContent { min: 8, max: 20 };

#[derive(Clone, Debug)]
pub struct WidgetGalleryApp {
    widget: String,
}

impl WidgetGalleryApp {
    pub fn new(widget: impl Into<String>) -> Self {
        Self {
            widget: widget.into(),
        }
    }
}

impl App for WidgetGalleryApp {
    type Message = ();

    fn render(
        &mut self,
        _cx: &mut ansiq_core::ViewCtx<'_, Self::Message>,
    ) -> Element<Self::Message> {
        widget_demo(&self.widget)
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

pub fn known_widgets() -> &'static [&'static str] {
    &[
        "bar-chart",
        "block",
        "bottom-pane",
        "box",
        "canvas",
        "chart",
        "clear",
        "composer-bar",
        "gauge",
        "input",
        "line-gauge",
        "list",
        "monthly",
        "pane",
        "paragraph",
        "rich-text",
        "scroll-view",
        "scrollbar",
        "session-header",
        "session-transcript",
        "shell",
        "sparkline",
        "status-bar",
        "streaming-text",
        "table",
        "tabs",
        "text",
        "transcript-view",
    ]
}

fn widget_demo(name: &str) -> Element<()> {
    match name {
        "bar-chart" => bar_chart_demo(),
        "block" => block_demo(),
        "bottom-pane" => bottom_pane_demo(),
        "box" => box_demo(),
        "canvas" => canvas_demo(),
        "chart" => chart_demo(),
        "clear" => clear_demo(),
        "composer-bar" => composer_bar_demo(),
        "gauge" => gauge_demo(),
        "input" => input_demo(),
        "line-gauge" => line_gauge_demo(),
        "list" => list_demo(),
        "monthly" => monthly_demo(),
        "pane" => pane_demo(),
        "paragraph" => paragraph_demo(),
        "rich-text" => rich_text_demo(),
        "scroll-view" => scroll_view_demo(),
        "scrollbar" => scrollbar_demo(),
        "session-header" => session_header_demo(),
        "session-transcript" => session_transcript_demo(),
        "shell" => shell_demo(),
        "sparkline" => sparkline_demo(),
        "status-bar" => status_bar_demo(),
        "streaming-text" => streaming_text_demo(),
        "table" => table_demo(),
        "tabs" => tabs_demo(),
        "text" => text_demo(),
        "transcript-view" => transcript_view_demo(),
        other => fallback_demo(other),
    }
}

fn fallback_demo(name: &str) -> Element<()> {
    Box::column()
        .gap(1)
        .child(StatusBar::new(format!("> widget gallery · {name}")).build())
        .child(
            Paragraph::new(format!("Unknown widget `{name}`"))
                .block(Block::bordered().title("Widget").padding(Padding::all(1)))
                .build(),
        )
        .build()
}

fn text_demo() -> Element<()> {
    Text::new("Ready").build()
}

fn paragraph_demo() -> Element<()> {
    Paragraph::new("Ansiq keeps a retained UI tree.")
        .wrap(Wrap { trim: true })
        .build()
}

fn block_demo() -> Element<()> {
    Block::bordered()
        .title("Server")
        .padding(Padding::all(1))
        .child(
            Paragraph::new("Ready\n2 healthy workers")
                .wrap(Wrap { trim: true })
                .build(),
        )
        .build()
}

fn box_demo() -> Element<()> {
    Box::column()
        .gap(1)
        .child(Text::new("Header").build())
        .child(Paragraph::new("Body").build())
        .build()
}

fn pane_demo() -> Element<()> {
    Pane::new()
        .title("Details")
        .child(Paragraph::new("Selected process").build())
        .build()
}

fn shell_demo() -> Element<()> {
    Shell::new()
        .header(StatusBar::new("ansiq · ready").build())
        .body(Paragraph::new("Main area").build())
        .footer(Input::new().placeholder("Type here").build())
        .build()
}

fn clear_demo() -> Element<()> {
    Box::column()
        .child(
            Clear::new()
                .layout(Layout {
                    width: Length::Fill,
                    height: Length::Fixed(2),
                })
                .build(),
        )
        .child(Pane::new().title("Modal").build())
        .build()
}

fn rich_text_demo() -> Element<()> {
    let block = HistoryBlock {
        lines: vec![
            HistoryLine {
                runs: vec![HistoryRun {
                    text: "line 1".to_string(),
                    style: Style::default(),
                }],
            },
            HistoryLine {
                runs: vec![HistoryRun {
                    text: "line 2".to_string(),
                    style: Style::default(),
                }],
            },
        ],
    };

    RichText::new(block)
        .layout(Layout {
            width: Length::Fill,
            height: Length::Auto,
        })
        .build()
}

fn streaming_text_demo() -> Element<()> {
    StreamingText::new("chunk 1\nchunk 2\nchunk 3").build()
}

fn input_demo() -> Element<()> {
    Input::new().value("").placeholder("Type a command").build()
}

fn list_demo() -> Element<()> {
    List::new(["Inbox", "Today", "Done"])
        .selected(Some(0))
        .highlight_symbol("> ")
        .highlight_spacing(ansiq_core::HighlightSpacing::Always)
        .build()
}

fn tabs_demo() -> Element<()> {
    Tabs::new(["CPU", "Memory", "Network"])
        .selected(Some(0))
        .divider("|")
        .build()
}

fn table_demo() -> Element<()> {
    let rows = vec![
        Row::new(["ansiq", "ready"]),
        Row::new(["monitor", "streaming"]),
    ];

    Table::new(
        rows,
        [Constraint::Percentage(60), Constraint::Percentage(40)],
    )
    .header(Row::new(["Name", "Status"]))
    .selected(Some(0))
    .build()
}

fn scroll_view_demo() -> Element<()> {
    ScrollView::new()
        .offset(2)
        .layout(Layout {
            width: Length::Fill,
            height: Length::Fixed(4),
        })
        .child(
            Paragraph::new("line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8")
                .build(),
        )
        .build()
}

fn scrollbar_demo() -> Element<()> {
    Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .content_length(200)
        .viewport_length(20)
        .position(40)
        .layout(Layout {
            width: Length::Fixed(1),
            height: Length::Fixed(8),
        })
        .build()
}

fn gauge_demo() -> Element<()> {
    Gauge::new().percent(72).label("72%").build()
}

fn line_gauge_demo() -> Element<()> {
    LineGauge::new()
        .ratio(0.63)
        .label(Line::from("CPU"))
        .build()
}

fn sparkline_demo() -> Element<()> {
    Sparkline::new().values([3, 5, 2, 8, 6, 9]).max(10).build()
}

fn bar_chart_demo() -> Element<()> {
    BarChart::new()
        .bar("Mon", 12)
        .bar("Tue", 18)
        .max(20)
        .layout(Layout {
            width: Length::Fill,
            height: Length::Fixed(6),
        })
        .build()
}

fn chart_demo() -> Element<()> {
    Chart::new()
        .named_dataset("cpu", [(0, 1), (1, 3), (2, 2)])
        .min_y(0)
        .max_y(5)
        .layout(Layout {
            width: Length::Fill,
            height: Length::Fixed(6),
        })
        .build()
}

fn canvas_demo() -> Element<()> {
    Canvas::new()
        .size(20, 8)
        .point(2, 2, '•')
        .point(10, 5, '•')
        .layout(Layout {
            width: Length::Fill,
            height: Length::Fixed(8),
        })
        .build()
}

fn monthly_demo() -> Element<()> {
    Monthly::new()
        .year(2026)
        .month(4)
        .selected_day(7)
        .layout(Layout {
            width: Length::Fill,
            height: Length::Fixed(8),
        })
        .build()
}

fn status_bar_demo() -> Element<()> {
    StatusBar::new("ansiq · ready").build()
}

fn bottom_pane_demo() -> Element<()> {
    BottomPane::new()
        .composer(Input::new().placeholder("Ask").build())
        .footer(Text::new("Esc to cancel").build())
        .build()
}

fn composer_bar_demo() -> Element<()> {
    ComposerBar::new()
        .value("")
        .placeholder("Type a request")
        .meta("Esc to cancel")
        .build()
}

fn session_header_demo() -> Element<()> {
    SessionHeader::new()
        .status("ansiq · idle")
        .title("OpenAPI Explorer")
        .meta_line("source: petstore.yaml")
        .build()
}

fn session_transcript_demo() -> Element<()> {
    SessionTranscript::new()
        .session(TranscriptSession::default())
        .intro(Paragraph::new("Start typing").build())
        .empty(Text::new("No output yet").build())
        .build()
}

fn transcript_view_demo() -> Element<()> {
    TranscriptView::new([
        TranscriptEntry::new(TranscriptRole::User, "hello"),
        TranscriptEntry::new(TranscriptRole::Assistant, "hi"),
    ])
    .build()
}

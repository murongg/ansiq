use std::sync::{Arc, Mutex};

use ansiq_core::{
    Alignment, BlockTitle, BorderType, Borders, Cell, Constraint, Direction, Element, ElementKind,
    HighlightSpacing, HistoryBlock, HistoryLine, HistoryRun, Length, Line, ListItem, ListState,
    Padding, Row, ScrollDirection, ScrollbarState, Span, Style, TableAlignment, TableState,
    TitlePosition, TranscriptEntry, TranscriptSession, Wrap,
};
use ansiq_widgets::{
    BarChart, Block, BottomPane, Box, Canvas, Chart, Clear, ComposerBar, Gauge, Input, LineGauge,
    List, Monthly, Pane, Paragraph, RichText, ScrollView, Scrollbar, SessionHeader,
    SessionTranscript, Shell, Sparkline, StatusBar, StreamingText, Table, Tabs, Text,
};

#[test]
fn column_builder_collects_children_and_gap() {
    let element: Element<()> = Box::column()
        .gap(2)
        .child(Text::new("Ansiq").build())
        .child(StatusBar::new("ready").build())
        .build();

    match &element.kind {
        ElementKind::Box(props) => {
            assert_eq!(props.direction, Direction::Column);
            assert_eq!(props.gap, 2);
        }
        other => panic!("expected box, got {other:?}"),
    }

    assert_eq!(element.layout.width, Length::Fill);
    assert_eq!(element.layout.height, Length::Fill);
    assert_eq!(element.children.len(), 2);
}

#[test]
fn input_builder_sets_focus_and_handlers() {
    let changes = Arc::new(Mutex::new(Vec::new()));
    let submits = Arc::new(Mutex::new(Vec::new()));

    let input = Input::new()
        .value("hello")
        .placeholder("Type here")
        .on_change({
            let changes = Arc::clone(&changes);
            move |value| changes.lock().unwrap().push(value)
        })
        .on_submit({
            let submits = Arc::clone(&submits);
            move |value| {
                submits.lock().unwrap().push(value.clone());
                Some(value.len())
            }
        })
        .build();

    assert!(input.focusable);
    assert_eq!(input.layout.width, Length::Fill);
    assert_eq!(input.layout.height, Length::Fixed(3));

    match input.kind {
        ElementKind::Input(mut props) => {
            assert_eq!(props.value, "hello");
            assert_eq!(props.placeholder, "Type here");

            props.on_change.as_mut().unwrap()("next".to_string());
            assert_eq!(&*changes.lock().unwrap(), &["next".to_string()]);

            let message = props.on_submit.as_mut().unwrap()("submit".to_string());
            assert_eq!(message, Some(6));
            assert_eq!(&*submits.lock().unwrap(), &["submit".to_string()]);
        }
        other => panic!("expected input, got {other:?}"),
    }
}

#[test]
fn pane_scrollview_and_streaming_text_build_nested_elements() {
    let element: Element<usize> = Pane::new()
        .title("Output")
        .child(
            ScrollView::new()
                .follow_bottom(true)
                .offset(2)
                .on_scroll(|position| Some(position))
                .child(StreamingText::new("streaming").build())
                .build(),
        )
        .build();

    match &element.kind {
        ElementKind::Pane(props) => assert_eq!(props.title.as_deref(), Some("Output")),
        other => panic!("expected pane, got {other:?}"),
    }

    assert_eq!(element.children.len(), 1);
    match &element.children[0].kind {
        ElementKind::ScrollView(props) => {
            assert!(props.follow_bottom);
            assert_eq!(props.offset, Some(2));
            assert_eq!(props.on_scroll.as_ref().map(|_| true), Some(true));
        }
        other => panic!("expected scroll view, got {other:?}"),
    }

    assert!(element.children[0].focusable);

    match &element.children[0].children[0].kind {
        ElementKind::StreamingText(props) => assert_eq!(props.content, "streaming"),
        other => panic!("expected streaming text, got {other:?}"),
    }
}

#[test]
fn text_and_status_bar_apply_default_layouts() {
    let text: Element<()> = Text::new("hello").build();
    let status: Element<()> = StatusBar::new("ready").build();

    assert_eq!(text.layout.width, Length::Fill);
    assert_eq!(text.layout.height, Length::Auto);
    assert_eq!(status.layout.width, Length::Fill);
    assert_eq!(status.layout.height, Length::Fixed(1));
}

#[test]
fn block_paragraph_and_list_build_expected_elements() {
    let block: Element<()> = Block::bordered()
        .title("Files")
        .border_type(BorderType::Rounded)
        .padding(Padding::all(1))
        .child(Paragraph::new("src\nCargo.toml").build())
        .build();
    let list: Element<()> = List::new([ListItem::new("src"), ListItem::new("Cargo.toml")])
        .selected(Some(1))
        .build();

    match &block.kind {
        ElementKind::Block(props) => {
            assert_eq!(props.titles, vec![BlockTitle::new("Files")]);
            assert_eq!(props.title_alignment, Alignment::Left);
            assert_eq!(props.title_position, TitlePosition::Top);
            assert_eq!(props.borders, Borders::ALL);
            assert_eq!(props.border_type, BorderType::Rounded);
            assert_eq!(props.padding, Padding::all(1));
        }
        other => panic!("expected block, got {other:?}"),
    }

    match &block.children[0].kind {
        ElementKind::Paragraph(props) => {
            assert_eq!(props.content.height(), 2);
            assert_eq!(props.block, None);
            assert_eq!(props.alignment, Alignment::Left);
            assert_eq!(props.wrap, None);
            assert_eq!(props.scroll_x, 0);
            assert_eq!(props.scroll_y, 0);
        }
        other => panic!("expected paragraph, got {other:?}"),
    }

    match &list.kind {
        ElementKind::List(props) => {
            assert_eq!(
                props.items,
                vec![ListItem::new("src"), ListItem::new("Cargo.toml")]
            );
            assert_eq!(props.state.selected(), Some(1));
            assert_eq!(props.state.offset(), 0);
            assert_eq!(props.highlight_symbol, None);
            assert_eq!(props.highlight_spacing, HighlightSpacing::WhenSelected);
        }
        other => panic!("expected list, got {other:?}"),
    }
}

#[test]
fn paragraph_builder_sets_alignment_wrap_and_scroll() {
    let paragraph_builder = Paragraph::new(Line::from(vec![
        Span::raw("one"),
        Span::raw(" "),
        Span::styled("two", Style::default().fg(ansiq_core::Color::Cyan)),
        Span::raw(" "),
        Span::raw("three"),
    ]))
    .block(Block::bordered().title("Paragraph"))
    .centered()
    .wrap(Wrap { trim: false })
    .scroll((3, 2));

    assert_eq!(paragraph_builder.line_width(), 15);
    assert_eq!(paragraph_builder.line_count(8), 4);

    let paragraph: Element<()> = paragraph_builder.build();

    match &paragraph.kind {
        ElementKind::Paragraph(props) => {
            assert!(props.block.is_some());
            assert_eq!(props.content.height(), 1);
            assert_eq!(props.content.width(), 13);
            assert_eq!(props.alignment, Alignment::Center);
            assert_eq!(props.wrap, Some(Wrap { trim: false }));
            assert_eq!(props.scroll_y, 3);
            assert_eq!(props.scroll_x, 2);
        }
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn block_new_defaults_to_no_borders_but_bordered_enables_them() {
    let plain: Element<()> = Block::new().title("Plain").build();
    let bordered: Element<()> = Block::bordered().title("Bordered").build();
    let custom: Element<()> = Block::new()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_type(BorderType::Double)
        .title("Custom")
        .build();

    match &plain.kind {
        ElementKind::Block(props) => {
            assert_eq!(props.borders, Borders::NONE);
            assert_eq!(props.titles, vec![BlockTitle::new("Plain")]);
        }
        other => panic!("expected block, got {other:?}"),
    }

    match &bordered.kind {
        ElementKind::Block(props) => {
            assert_eq!(props.borders, Borders::ALL);
            assert_eq!(props.border_type, BorderType::Plain);
        }
        other => panic!("expected block, got {other:?}"),
    }

    match &custom.kind {
        ElementKind::Block(props) => {
            assert_eq!(props.titles, vec![BlockTitle::new("Custom")]);
            assert_eq!(props.borders, Borders::LEFT | Borders::RIGHT);
            assert_eq!(props.border_type, BorderType::Double);
        }
        other => panic!("expected block, got {other:?}"),
    }
}

#[test]
fn block_supports_ratatui_like_title_positions_and_alignment_defaults() {
    let block: Element<()> = Block::bordered()
        .title_alignment(Alignment::Center)
        .title_position(TitlePosition::Bottom)
        .title("default")
        .title_top(Line::from("top").left_aligned())
        .title_bottom(Line::from("bottom").right_aligned())
        .build();

    match &block.kind {
        ElementKind::Block(props) => {
            assert_eq!(props.title_alignment, Alignment::Center);
            assert_eq!(props.title_position, TitlePosition::Bottom);
            assert_eq!(
                props.titles,
                vec![
                    BlockTitle::new("default"),
                    BlockTitle::top(Line::from("top").left_aligned()),
                    BlockTitle::bottom(Line::from("bottom").right_aligned()),
                ]
            );
        }
        other => panic!("expected block, got {other:?}"),
    }
}

#[test]
fn block_supports_custom_border_set_like_ratatui() {
    let block: Element<()> = Block::bordered()
        .title("Custom")
        .border_set(ansiq_core::symbols::border::Set {
            top_left: 'A',
            top_right: 'B',
            bottom_left: 'C',
            bottom_right: 'D',
            horizontal_top: '=',
            horizontal_bottom: '_',
            vertical_left: '!',
            vertical_right: '?',
        })
        .build();

    match &block.kind {
        ElementKind::Block(props) => {
            assert_eq!(
                props.border_set,
                Some(ansiq_core::symbols::border::Set {
                    top_left: 'A',
                    top_right: 'B',
                    bottom_left: 'C',
                    bottom_right: 'D',
                    horizontal_top: '=',
                    horizontal_bottom: '_',
                    vertical_left: '!',
                    vertical_right: '?',
                })
            );
        }
        other => panic!("expected block, got {other:?}"),
    }
}

#[test]
fn block_inner_matches_ratatui_like_border_and_padding_insets() {
    let block: Block<()> = Block::bordered()
        .title("Processes")
        .padding(Padding::all(1));
    let inner = block.inner(ansiq_core::Rect::new(0, 0, 20, 5));

    assert_eq!(inner, ansiq_core::Rect::new(2, 2, 16, 1));
}

#[test]
fn list_and_tabs_can_wrap_themselves_in_blocks() {
    let list: Element<()> = List::new(["src", "Cargo.toml"])
        .block(Block::bordered().title("List"))
        .build();
    let tabs: Element<()> = Tabs::new(["CPU", "Memory"])
        .block(Block::bordered().title("Tabs"))
        .build();

    match &list.kind {
        ElementKind::List(props) => assert!(props.block.is_some()),
        other => panic!("expected list, got {other:?}"),
    }

    match &tabs.kind {
        ElementKind::Tabs(props) => assert!(props.block.is_some()),
        other => panic!("expected tabs, got {other:?}"),
    }
}

#[test]
fn tabs_and_gauge_build_expected_elements() {
    let tabs: Element<()> = Tabs::new([
        Line::from("Overview"),
        Line::from(vec![
            Span::raw("Fi"),
            Span::styled("les", Style::default().fg(ansiq_core::Color::Yellow)),
        ]),
    ])
    .select(1)
    .divider(Span::raw("•"))
    .padding(Line::from(" "), Line::from("  "))
    .highlight_style(Style::default().fg(ansiq_core::Color::Yellow))
    .build();
    let gauge: Element<()> = Gauge::new()
        .percent(42)
        .label(Span::styled(
            "42%",
            Style::default().fg(ansiq_core::Color::Cyan),
        ))
        .use_unicode(true)
        .gauge_style(Style::default().fg(ansiq_core::Color::Cyan))
        .build();

    match &tabs.kind {
        ElementKind::Tabs(props) => {
            assert_eq!(
                props.titles,
                vec![
                    Line::from("Overview"),
                    Line::from(vec![
                        Span::raw("Fi"),
                        Span::styled("les", Style::default().fg(ansiq_core::Color::Yellow)),
                    ]),
                ]
            );
            assert_eq!(props.selected, Some(1));
            assert_eq!(props.divider, Span::raw("•"));
            assert_eq!(props.padding_left, Line::from(" "));
            assert_eq!(props.padding_right, Line::from("  "));
            assert_eq!(props.highlight_style.fg, ansiq_core::Color::Yellow);
        }
        other => panic!("expected tabs, got {other:?}"),
    }
    assert_eq!(tabs.layout.height, Length::Fixed(1));

    match &gauge.kind {
        ElementKind::Gauge(props) => {
            assert!(props.block.is_none());
            assert_eq!(props.ratio, 0.42);
            assert_eq!(
                props.label,
                Some(Span::styled(
                    "42%",
                    Style::default().fg(ansiq_core::Color::Cyan),
                ))
            );
            assert!(props.use_unicode);
            assert_eq!(props.gauge_style.fg, ansiq_core::Color::Cyan);
        }
        other => panic!("expected gauge, got {other:?}"),
    }
    assert_eq!(gauge.layout.height, Length::Auto);
}

#[test]
fn tabs_default_to_a_pipe_divider_and_select_the_first_title() {
    let tabs: Element<()> = Tabs::new(["CPU", "Memory"]).build();

    match &tabs.kind {
        ElementKind::Tabs(props) => {
            assert_eq!(props.selected, Some(0));
            assert_eq!(
                props.divider,
                Span::raw(ansiq_core::symbols::line::VERTICAL)
            );
            assert_eq!(props.padding_left, Line::from(" "));
            assert_eq!(props.padding_right, Line::from(" "));
        }
        other => panic!("expected tabs, got {other:?}"),
    }
}

#[test]
fn tabs_collect_from_iterator_like_ratatui() {
    let tabs: Tabs = ["CPU", "Memory", "Disk"].into_iter().collect();
    let element: Element<()> = tabs.build();

    match &element.kind {
        ElementKind::Tabs(props) => {
            assert_eq!(
                props.titles,
                vec![Line::from("CPU"), Line::from("Memory"), Line::from("Disk")]
            );
            assert_eq!(props.selected, Some(0));
        }
        other => panic!("expected tabs, got {other:?}"),
    }
}

#[test]
fn list_and_scrollbar_capture_extended_state() {
    let list: Element<()> = List::new([
        ListItem::new("alpha"),
        ListItem::new("beta"),
        ListItem::new("gamma"),
    ])
    .state(ListState::default().with_selected(Some(2)).with_offset(1))
    .highlight_symbol(">>")
    .highlight_spacing(HighlightSpacing::Always)
    .build();
    let scrollbar: Element<()> = Scrollbar::new(ansiq_core::ScrollbarOrientation::HorizontalBottom)
        .state(
            ansiq_core::ScrollbarState::new(30)
                .position(4)
                .viewport_content_length(10),
        )
        .symbols(ansiq_core::symbols::scrollbar::HORIZONTAL)
        .begin_symbol(None::<String>)
        .build();

    match &list.kind {
        ElementKind::List(props) => {
            assert_eq!(props.state.selected(), Some(2));
            assert_eq!(props.state.offset(), 1);
            assert_eq!(props.highlight_symbol, Some(Line::from(">>")));
            assert_eq!(props.highlight_spacing, HighlightSpacing::Always);
            assert_eq!(props.items[0].height(), 1);
        }
        other => panic!("expected list, got {other:?}"),
    }

    match &scrollbar.kind {
        ElementKind::Scrollbar(props) => {
            assert_eq!(
                props.orientation,
                ansiq_core::ScrollbarOrientation::HorizontalBottom
            );
            assert_eq!(
                props.state,
                ansiq_core::ScrollbarState::new(30)
                    .position(4)
                    .viewport_content_length(10)
            );
            assert_eq!(props.begin_symbol.as_deref(), None);
            assert_eq!(props.end_symbol.as_deref(), Some("→"));
            assert_eq!(props.track_symbol.as_deref(), Some("─"));
            assert_eq!(props.thumb_symbol, "█");
        }
        other => panic!("expected scrollbar, got {other:?}"),
    }
}

#[test]
fn list_item_tracks_multiline_content_metrics() {
    let item = ListItem::new("alpha\nbeta").style(Style::default().fg(ansiq_core::Color::Yellow));

    assert_eq!(item.height(), 2);
    assert_eq!(item.width(), 5);
}

#[test]
fn shell_builder_assigns_header_body_and_footer_slots() {
    let shell: Element<()> = Shell::new()
        .header(StatusBar::new("ready").build())
        .body(StreamingText::new("body").build())
        .footer(Input::new().placeholder("prompt").build())
        .build();

    match &shell.kind {
        ElementKind::Shell(_) => {}
        other => panic!("expected shell, got {other:?}"),
    }

    assert_eq!(shell.children.len(), 3);
    assert!(matches!(shell.children[0].kind, ElementKind::StatusBar(_)));
    assert!(matches!(
        shell.children[1].kind,
        ElementKind::StreamingText(_)
    ));
    assert!(matches!(shell.children[2].kind, ElementKind::Input(_)));
}

#[test]
fn session_header_builds_a_status_bar_and_banner_pane() {
    let header: Element<()> = SessionHeader::new()
        .status("> ansiq · idle")
        .title("OpenAI Codex")
        .meta_line("model: gpt-5.4 xhigh")
        .meta_line("directory: /tmp/ansiq")
        .build();

    match &header.kind {
        ElementKind::Box(props) => {
            assert_eq!(props.direction, Direction::Column);
            assert_eq!(props.gap, 1);
        }
        other => panic!("expected box, got {other:?}"),
    }

    assert_eq!(header.children.len(), 2);
    assert!(matches!(header.children[0].kind, ElementKind::StatusBar(_)));
    assert!(matches!(header.children[1].kind, ElementKind::Pane(_)));
}

#[test]
fn bottom_pane_builder_stacks_composer_and_footer() {
    let bottom: Element<()> = BottomPane::new()
        .composer(Input::new().placeholder("prompt").build())
        .footer(Text::new("footer").build())
        .build();

    match &bottom.kind {
        ElementKind::Box(props) => {
            assert_eq!(props.direction, Direction::Column);
            assert_eq!(props.gap, 0);
        }
        other => panic!("expected box, got {other:?}"),
    }

    assert_eq!(bottom.layout.width, Length::Fill);
    assert_eq!(bottom.layout.height, Length::Auto);
    assert_eq!(bottom.children.len(), 2);
    assert!(matches!(bottom.children[0].kind, ElementKind::Input(_)));
    assert!(matches!(bottom.children[1].kind, ElementKind::Text(_)));
}

#[test]
fn composer_bar_builds_input_and_meta_text() {
    let composer: Element<()> = ComposerBar::new()
        .value("cargo test")
        .placeholder("Write tests")
        .meta("gpt-5.4 xhigh · ready")
        .build();

    match &composer.kind {
        ElementKind::Box(props) => {
            assert_eq!(props.direction, Direction::Column);
            assert_eq!(props.gap, 0);
        }
        other => panic!("expected box, got {other:?}"),
    }

    assert_eq!(composer.children.len(), 2);
    assert!(matches!(composer.children[0].kind, ElementKind::Input(_)));
    assert!(matches!(composer.children[1].kind, ElementKind::Text(_)));
}

#[test]
fn session_transcript_switches_between_intro_active_and_empty_states() {
    let onboarding: Element<()> = SessionTranscript::new()
        .session_started(false)
        .intro(
            Pane::new()
                .title("Welcome")
                .child(Text::new("hello").build())
                .build(),
        )
        .active(StreamingText::new("assistant").build())
        .empty(Text::new("idle").build())
        .build();

    let active: Element<()> = SessionTranscript::new()
        .session_started(true)
        .intro(
            Pane::new()
                .title("Welcome")
                .child(Text::new("hello").build())
                .build(),
        )
        .active(StreamingText::new("assistant").build())
        .empty(Text::new("idle").build())
        .build();

    let idle: Element<()> = SessionTranscript::new()
        .session_started(true)
        .intro(
            Pane::new()
                .title("Welcome")
                .child(Text::new("hello").build())
                .build(),
        )
        .empty(Text::new("idle").build())
        .build();

    match &onboarding.kind {
        ElementKind::Box(props) => {
            assert_eq!(props.direction, Direction::Column);
            assert_eq!(props.gap, 1);
        }
        other => panic!("expected box, got {other:?}"),
    }
    assert_eq!(onboarding.children.len(), 1);
    assert!(matches!(onboarding.children[0].kind, ElementKind::Pane(_)));

    match &active.kind {
        ElementKind::Box(props) => {
            assert_eq!(props.direction, Direction::Column);
            assert_eq!(props.gap, 1);
        }
        other => panic!("expected box, got {other:?}"),
    }
    assert_eq!(active.children.len(), 1);
    assert!(matches!(
        active.children[0].kind,
        ElementKind::StreamingText(_)
    ));

    match &idle.kind {
        ElementKind::Box(props) => {
            assert_eq!(props.direction, Direction::Column);
            assert_eq!(props.gap, 1);
        }
        other => panic!("expected box, got {other:?}"),
    }
    assert_eq!(idle.children.len(), 1);
    assert!(matches!(idle.children[0].kind, ElementKind::Text(_)));
}

#[test]
fn session_transcript_can_render_structured_entries_without_manual_rich_text() {
    let active: Element<()> = SessionTranscript::new()
        .session_started(true)
        .entries([
            TranscriptEntry::user("cargo test"),
            TranscriptEntry::assistant("Planning the change"),
        ])
        .empty(Text::new("idle").build())
        .build();

    match &active.kind {
        ElementKind::Box(props) => {
            assert_eq!(props.direction, Direction::Column);
            assert_eq!(props.gap, 1);
        }
        other => panic!("expected box, got {other:?}"),
    }

    assert_eq!(active.children.len(), 1);
    assert!(matches!(active.children[0].kind, ElementKind::RichText(_)));
}

#[test]
fn session_transcript_can_render_a_transcript_session_directly() {
    let mut session = TranscriptSession::default();
    session.begin_turn("cargo test");
    session.append_assistant("Planning the change");

    let active: Element<()> = SessionTranscript::new()
        .session(session)
        .intro(
            Pane::new()
                .title("Welcome")
                .child(Text::new("hello").build())
                .build(),
        )
        .empty(Text::new("idle").build())
        .build();

    match &active.kind {
        ElementKind::Box(props) => {
            assert_eq!(props.direction, Direction::Column);
            assert_eq!(props.gap, 1);
        }
        other => panic!("expected box, got {other:?}"),
    }

    assert_eq!(active.children.len(), 1);
    assert!(matches!(active.children[0].kind, ElementKind::RichText(_)));
}

#[test]
fn rich_text_builder_wraps_a_history_block() {
    let block = HistoryBlock {
        lines: vec![HistoryLine {
            runs: vec![HistoryRun {
                text: "assistant  Planning".to_string(),
                style: Default::default(),
            }],
        }],
    };

    let rich: Element<()> = RichText::new(block.clone()).build();

    match &rich.kind {
        ElementKind::RichText(props) => assert_eq!(props.block, block),
        other => panic!("expected rich text, got {other:?}"),
    }
    assert_eq!(rich.layout.width, Length::Fill);
    assert_eq!(rich.layout.height, Length::Auto);
}

#[test]
fn interactive_selection_widgets_become_focusable_and_store_handlers() {
    let list = List::new(["src", "Cargo.toml"])
        .on_select(|index| Some(index))
        .build();
    let tabs = Tabs::new(["Overview", "Files"])
        .on_select(|index| Some(index))
        .build();
    let table = Table::new(
        [["ansiq", "ready"], ["codexdemo", "streaming"]],
        [Constraint::Fill(1), Constraint::Fill(1)],
    )
    .header(["Name", "Status"])
    .on_select(|index| Some(index))
    .build();

    assert!(list.focusable);
    assert!(tabs.focusable);
    assert!(table.focusable);

    match list.kind {
        ElementKind::List(mut props) => {
            assert_eq!(props.on_select.as_mut().unwrap()(1), Some(1));
        }
        other => panic!("expected list, got {other:?}"),
    }

    match tabs.kind {
        ElementKind::Tabs(mut props) => {
            assert_eq!(props.on_select.as_mut().unwrap()(1), Some(1));
        }
        other => panic!("expected tabs, got {other:?}"),
    }

    match table.kind {
        ElementKind::Table(mut props) => {
            assert_eq!(props.on_select.as_mut().unwrap()(1), Some(1));
        }
        other => panic!("expected table, got {other:?}"),
    }
}

#[test]
fn list_collects_from_iterator_like_ratatui() {
    let list: List = ["alpha", "beta", "gamma"].into_iter().collect();
    let element: Element<()> = list.build();

    match &element.kind {
        ElementKind::List(props) => {
            assert_eq!(
                props.items,
                vec![
                    ListItem::new("alpha"),
                    ListItem::new("beta"),
                    ListItem::new("gamma")
                ]
            );
        }
        other => panic!("expected list, got {other:?}"),
    }
}

#[test]
fn list_item_style_accepts_into_style_like_ratatui() {
    let item = ListItem::new("alpha").style(ansiq_core::Color::Cyan);
    assert_eq!(item.style.fg, ansiq_core::Color::Cyan);
}

#[test]
fn table_builds_expected_headers_rows_and_selection() {
    let table: Element<()> = Table::new(
        [["ansiq", "ready"], ["codexdemo", "streaming"]],
        [Constraint::Length(10), Constraint::Length(12)],
    )
    .header(["Name", "Status"])
    .footer(["Summary", "2 rows"])
    .block(Block::bordered().title("Processes"))
    .column_spacing(2)
    .flex(ansiq_core::Flex::Start)
    .row_highlight_style(Style::default().bold(true))
    .alignments([TableAlignment::Left, TableAlignment::Right])
    .state(TableState::new().with_selected(Some(1)).with_offset(2))
    .highlight_symbol(">>")
    .highlight_spacing(HighlightSpacing::Always)
    .build();

    match &table.kind {
        ElementKind::Table(props) => {
            assert_eq!(
                props.header.as_ref().unwrap(),
                &Row::new(["Name", "Status"])
            );
            assert_eq!(
                props.footer.as_ref().unwrap(),
                &Row::new(["Summary", "2 rows"])
            );
            assert_eq!(
                props.rows,
                vec![
                    Row::new(["ansiq", "ready"]),
                    Row::new(["codexdemo", "streaming"]),
                ]
            );
            assert!(props.block.is_some());
            assert_eq!(
                props.widths,
                vec![Constraint::Length(10), Constraint::Length(12)]
            );
            assert_eq!(props.column_spacing, 2);
            assert_eq!(props.flex, ansiq_core::Flex::Start);
            assert_eq!(
                props.alignments,
                vec![TableAlignment::Left, TableAlignment::Right]
            );
            assert_eq!(props.state.selected(), Some(1));
            assert_eq!(props.state.offset(), 2);
            assert_eq!(props.highlight_symbol, Some(">>".into()));
            assert_eq!(props.highlight_spacing, HighlightSpacing::Always);
            assert!(props.row_highlight_style.bold);
        }
        other => panic!("expected table, got {other:?}"),
    }

    assert_eq!(table.layout.width, Length::Fill);
    assert_eq!(table.layout.height, Length::Auto);
}

#[test]
fn table_collects_rows_from_iterator_like_ratatui() {
    let table: Table = [
        Row::new(["ansiq", "ready"]),
        Row::new(["codexdemo", "streaming"]),
    ]
    .into_iter()
    .collect();
    let element: Element<()> = table.build();

    match &element.kind {
        ElementKind::Table(props) => {
            assert_eq!(
                props.rows,
                vec![
                    Row::new(["ansiq", "ready"]),
                    Row::new(["codexdemo", "streaming"]),
                ]
            );
            assert!(props.widths.is_empty());
        }
        other => panic!("expected table, got {other:?}"),
    }
}

#[test]
fn table_cells_and_rows_track_style_height_and_margins() {
    let cell = Cell::new(Line::from(vec![
        Span::raw("1"),
        Span::styled("23", Style::default().fg(ansiq_core::Color::Yellow)),
    ]))
    .style(Style::default().fg(ansiq_core::Color::Cyan))
    .column_span(2);
    let row = Row::new([
        Cell::new("cpu").style(Style::default().fg(ansiq_core::Color::Cyan)),
        cell.clone(),
    ])
    .height(2)
    .top_margin(1)
    .bottom_margin(1)
    .style(Style::default().bold(true));

    assert_eq!(cell.width(), 3);
    assert_eq!(cell.height(), 1);
    assert_eq!(cell.column_span_value(), 2);
    assert_eq!(cell.style_value().fg, ansiq_core::Color::Cyan);
    assert_eq!(
        row,
        Row::new([
            Cell::new("cpu").style(Style::default().fg(ansiq_core::Color::Cyan)),
            cell
        ])
        .height(2)
        .top_margin(1)
        .bottom_margin(1)
        .style(Style::default().bold(true))
    );
    assert_eq!(row.height_with_margin(), 4);
}

#[test]
fn scrollbar_builds_expected_track_metrics() {
    let scrollbar: Element<usize> = Scrollbar::new(ansiq_core::ScrollbarOrientation::VerticalRight)
        .state(
            ansiq_core::ScrollbarState::new(12)
                .position(6)
                .viewport_content_length(3),
        )
        .thumb_style(Style::default().fg(ansiq_core::Color::Yellow))
        .track_style(Style::default().fg(ansiq_core::Color::DarkGrey))
        .on_scroll(|position| Some(position))
        .build();

    match &scrollbar.kind {
        ElementKind::Scrollbar(props) => {
            assert_eq!(
                props.state,
                ansiq_core::ScrollbarState::new(12)
                    .position(6)
                    .viewport_content_length(3)
            );
            assert_eq!(props.thumb_style.fg, ansiq_core::Color::Yellow);
            assert_eq!(props.track_style.fg, ansiq_core::Color::DarkGrey);
            assert!(props.on_scroll.as_ref().is_some());
        }
        other => panic!("expected scrollbar, got {other:?}"),
    }

    assert!(scrollbar.focusable);
    assert_eq!(scrollbar.layout.width, Length::Fixed(1));
    assert_eq!(scrollbar.layout.height, Length::Fill);
}

#[test]
fn scrollbar_symbols_follow_ratatui_orientation_helpers() {
    let scrollbar: Element<()> = Scrollbar::new(ansiq_core::ScrollbarOrientation::VerticalRight)
        .begin_symbol(None::<String>)
        .orientation_and_symbol(
            ansiq_core::ScrollbarOrientation::HorizontalBottom,
            ansiq_core::symbols::scrollbar::HORIZONTAL,
        )
        .build();

    match &scrollbar.kind {
        ElementKind::Scrollbar(props) => {
            assert_eq!(
                props.orientation,
                ansiq_core::ScrollbarOrientation::HorizontalBottom
            );
            assert_eq!(props.begin_symbol, None);
            assert_eq!(props.end_symbol.as_deref(), Some("→"));
            assert_eq!(props.track_symbol.as_deref(), Some("─"));
            assert_eq!(props.thumb_symbol, "█");
        }
        other => panic!("expected scrollbar, got {other:?}"),
    }
}

#[test]
fn scrollbar_state_supports_ratatui_like_navigation_helpers() {
    let mut state = ScrollbarState::default()
        .content_length(10)
        .position(4)
        .viewport_content_length(3);

    assert_eq!(state.get_position(), 4);

    state.prev();
    assert_eq!(state.get_position(), 3);

    state.next();
    assert_eq!(state.get_position(), 4);

    state.scroll(ScrollDirection::Forward);
    assert_eq!(state.get_position(), 5);

    state.scroll(ScrollDirection::Backward);
    assert_eq!(state.get_position(), 4);

    state.first();
    assert_eq!(state.get_position(), 0);

    state.last();
    assert_eq!(state.get_position(), 9);
}

#[test]
fn scrollbar_state_supports_ratatui_like_with_builders() {
    let state = ScrollbarState::default()
        .with_content_length(10)
        .with_position(4)
        .with_viewport_content_length(3);

    assert_eq!(state.content_length_value(), 10);
    assert_eq!(state.get_position(), 4);
    assert_eq!(state.viewport_content_length_value(), 3);
}

#[test]
fn clear_and_line_gauge_build_expected_elements() {
    let clear: Element<()> = Clear::new().build();
    let line_gauge: Element<()> = LineGauge::new()
        .ratio(0.58)
        .label(Line::from("58%"))
        .line_set(ansiq_core::symbols::line::DOUBLE)
        .filled_style(Style::default().fg(ansiq_core::Color::Green))
        .unfilled_style(Style::default().fg(ansiq_core::Color::DarkGrey))
        .build();

    match &clear.kind {
        ElementKind::Clear(_) => {}
        other => panic!("expected clear, got {other:?}"),
    }
    assert_eq!(clear.layout.width, Length::Fill);
    assert_eq!(clear.layout.height, Length::Fill);

    match &line_gauge.kind {
        ElementKind::LineGauge(props) => {
            assert!(props.block.is_none());
            assert_eq!(props.ratio, 0.58);
            assert_eq!(props.label, Some(Line::from("58%")));
            assert_eq!(props.filled_symbol, "═");
            assert_eq!(props.unfilled_symbol, "═");
            assert_eq!(props.filled_style.fg, ansiq_core::Color::Green);
            assert_eq!(props.unfilled_style.fg, ansiq_core::Color::DarkGrey);
        }
        other => panic!("expected line gauge, got {other:?}"),
    }
    assert_eq!(line_gauge.layout.height, Length::Auto);
}

#[test]
fn sparkline_builds_expected_values_and_max() {
    let sparkline: Element<()> = Sparkline::new().values([1, 2, 4, 6, 8]).max(8).build();

    match &sparkline.kind {
        ElementKind::Sparkline(props) => {
            assert_eq!(
                props.values,
                vec![Some(1), Some(2), Some(4), Some(6), Some(8)]
            );
            assert_eq!(props.max, Some(8));
            assert_eq!(props.direction, ansiq_core::SparklineDirection::LeftToRight);
            assert_eq!(props.absent_value_symbol, '·');
        }
        other => panic!("expected sparkline, got {other:?}"),
    }

    assert_eq!(sparkline.layout.width, Length::Fill);
    assert_eq!(sparkline.layout.height, Length::Fixed(1));
}

#[test]
fn barchart_chart_canvas_and_monthly_build_expected_elements() {
    let barchart: Element<()> = BarChart::new()
        .bar("cpu", 64)
        .bar("mem", 42)
        .max(100)
        .bar_width(4)
        .build();
    let chart: Element<()> = Chart::new()
        .named_dataset("cpu", [(0, 1), (1, 3), (2, 2)])
        .min_y(0)
        .max_y(4)
        .build();
    let canvas: Element<()> = Canvas::new().size(10, 4).point(1, 1, '•').build();
    let monthly: Element<()> = Monthly::new().year(2026).month(4).selected_day(4).build();

    match &barchart.kind {
        ElementKind::BarChart(props) => {
            assert_eq!(props.bars.len(), 2);
            assert_eq!(props.bars[0].label, "cpu");
            assert_eq!(props.max, Some(100));
            assert_eq!(props.bar_width, 4);
        }
        other => panic!("expected bar chart, got {other:?}"),
    }
    assert_eq!(barchart.layout.height, Length::Fixed(6));

    match &chart.kind {
        ElementKind::Chart(props) => {
            assert_eq!(props.datasets.len(), 1);
            assert_eq!(props.datasets[0].label.as_deref(), Some("cpu"));
            assert_eq!(props.datasets[0].points, vec![(0, 1), (1, 3), (2, 2)]);
            assert_eq!(props.min_y, Some(0));
            assert_eq!(props.max_y, Some(4));
        }
        other => panic!("expected chart, got {other:?}"),
    }
    assert_eq!(chart.layout.height, Length::Fixed(8));

    match &canvas.kind {
        ElementKind::Canvas(props) => {
            assert_eq!(props.width, 10);
            assert_eq!(props.height, 4);
            assert_eq!(props.cells.len(), 1);
            assert_eq!(props.cells[0].symbol, '•');
        }
        other => panic!("expected canvas, got {other:?}"),
    }
    assert_eq!(canvas.layout.height, Length::Fixed(8));

    match &monthly.kind {
        ElementKind::Monthly(props) => {
            assert_eq!(props.year, 2026);
            assert_eq!(props.month, 4);
            assert_eq!(props.selected_day, Some(4));
        }
        other => panic!("expected monthly, got {other:?}"),
    }
    assert_eq!(monthly.layout.height, Length::Fixed(8));
}

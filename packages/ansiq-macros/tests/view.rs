use ansiq_core::{Element, ElementKind};
use ansiq_macros::view;

#[test]
fn view_macro_builds_basic_widget_tree() {
    let tree: Element<()> = view! {
        <Box direction="column">
            <Text content="hello" />
            <Input value={"world".to_string()} />
        </Box>
    };

    assert_eq!(tree.children.len(), 2);
}

#[test]
fn view_macro_supports_custom_components() {
    fn banner(cx: &mut ansiq_core::Cx<'_, ()>) -> Element<()> {
        let value = cx.signal(|| String::from("banner"));
        view! { <Text content={value.get()} /> }
    }

    let tree: Element<()> = view! { <banner /> };
    assert_eq!(tree.kind_name(), "Component");
}

#[test]
fn view_macro_supports_block_paragraph_and_list_primitives() {
    let tree: Element<()> = view! {
        <Block title="Files" bordered={true}>
            <Paragraph
                content="workspace"
                block={::ansiq_widgets::Block::bordered().title("Paragraph")}
                scroll={(1u16, 2u16)}
            />
            <List items={vec!["src".to_string(), "Cargo.toml".to_string()]} selected={Some(1usize)} />
        </Block>
    };

    match &tree.kind {
        ansiq_core::ElementKind::Block(props) => {
            assert_eq!(props.titles, vec![ansiq_core::BlockTitle::new("Files")]);
            assert_eq!(props.borders, ansiq_core::Borders::ALL);
            assert_eq!(props.border_type, ansiq_core::BorderType::Plain);
        }
        other => panic!("expected block, got {other:?}"),
    }

    match &tree.children[0].kind {
        ansiq_core::ElementKind::Paragraph(props) => {
            assert_eq!(props.content, "workspace".into());
            assert!(props.block.is_some());
            assert_eq!(props.scroll_y, 1);
            assert_eq!(props.scroll_x, 2);
        }
        other => panic!("expected paragraph, got {other:?}"),
    }

    match &tree.children[1].kind {
        ansiq_core::ElementKind::List(props) => {
            assert_eq!(props.items.len(), 2);
            assert!(props.block.is_none());
            assert_eq!(props.state.selected(), Some(1));
        }
        other => panic!("expected list, got {other:?}"),
    }
}

#[test]
fn view_macro_supports_block_borders_and_border_type() {
    let tree: Element<()> = view! {
        <Block
            title="Logs"
            borders={ansiq_core::Borders::LEFT | ansiq_core::Borders::RIGHT}
            border_type={ansiq_core::BorderType::Double}
        />
    };

    match &tree.kind {
        ansiq_core::ElementKind::Block(props) => {
            assert_eq!(props.titles, vec![ansiq_core::BlockTitle::new("Logs")]);
            assert_eq!(
                props.borders,
                ansiq_core::Borders::LEFT | ansiq_core::Borders::RIGHT
            );
            assert_eq!(props.border_type, ansiq_core::BorderType::Double);
        }
        other => panic!("expected block, got {other:?}"),
    }
}

#[test]
fn view_macro_supports_block_border_set() {
    let tree: Element<()> = view! {
        <Block
            title="Logs"
            border_set={ansiq_core::symbols::border::Set {
                top_left: 'A',
                top_right: 'B',
                bottom_left: 'C',
                bottom_right: 'D',
                horizontal_top: '=',
                horizontal_bottom: '_',
                vertical_left: '!',
                vertical_right: '?',
            }}
        />
    };

    match &tree.kind {
        ansiq_core::ElementKind::Block(props) => {
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
fn view_macro_supports_block_title_position_and_alignment() {
    let tree: Element<()> = view! {
        <Block
            title="Default"
            title_top={ansiq_core::Line::from("Top").left_aligned()}
            title_bottom={ansiq_core::Line::from("Bottom").right_aligned()}
            title_alignment={ansiq_core::Alignment::Center}
            title_position={ansiq_core::TitlePosition::Bottom}
        />
    };

    match &tree.kind {
        ansiq_core::ElementKind::Block(props) => {
            assert_eq!(props.title_alignment, ansiq_core::Alignment::Center);
            assert_eq!(props.title_position, ansiq_core::TitlePosition::Bottom);
            assert_eq!(props.titles[0], ansiq_core::BlockTitle::new("Default"));
            assert_eq!(
                props.titles[1],
                ansiq_core::BlockTitle::top(ansiq_core::Line::from("Top").left_aligned())
            );
            assert_eq!(
                props.titles[2],
                ansiq_core::BlockTitle::bottom(ansiq_core::Line::from("Bottom").right_aligned())
            );
        }
        other => panic!("expected block, got {other:?}"),
    }
}

#[test]
fn view_macro_supports_tabs_and_gauge_primitives() {
    let tree: Element<()> = view! {
        <Box direction="column">
            <Tabs
                titles={vec!["Overview".to_string(), "Files".to_string()]}
                selected={Some(1usize)}
                divider={ansiq_core::Span::raw("•")}
                padding_left={ansiq_core::Line::from(" ")}
                padding_right={ansiq_core::Line::from("  ")}
                highlight_style={ansiq_core::Style::default().fg(ansiq_core::Color::Yellow)}
            />
            <Gauge
                ratio={0.72f64}
                label={ansiq_core::Span::raw("72%")}
                use_unicode={true}
                block={::ansiq_widgets::Block::bordered().title("Progress")}
                gauge_style={ansiq_core::Style::default().fg(ansiq_core::Color::Cyan)}
            />
        </Box>
    };

    match &tree.children[0].kind {
        ansiq_core::ElementKind::Tabs(props) => {
            assert_eq!(props.titles.len(), 2);
            assert_eq!(props.titles[0], ansiq_core::Line::from("Overview"));
            assert!(props.block.is_none());
            assert_eq!(props.selected, Some(1));
            assert_eq!(props.divider, ansiq_core::Span::raw("•"));
            assert_eq!(props.padding_left, ansiq_core::Line::from(" "));
            assert_eq!(props.padding_right, ansiq_core::Line::from("  "));
            assert_eq!(props.highlight_style.fg, ansiq_core::Color::Yellow);
        }
        other => panic!("expected tabs, got {other:?}"),
    }

    match &tree.children[1].kind {
        ansiq_core::ElementKind::Gauge(props) => {
            assert!(props.block.is_some());
            assert_eq!(props.ratio, 0.72);
            assert_eq!(props.label, Some(ansiq_core::Span::raw("72%")));
            assert!(props.use_unicode);
        }
        other => panic!("expected gauge, got {other:?}"),
    }
}

#[test]
fn view_macro_supports_table_primitive() {
    let tree: Element<()> = view! {
        <Table
            header={vec!["Name".to_string(), "Status".to_string()]}
            footer={vec!["Summary".to_string(), "2 rows".to_string()]}
            rows={vec![
                vec!["ansiq".to_string(), "ready".to_string()],
                vec!["codexdemo".to_string(), "streaming".to_string()],
            ]}
            widths={vec![
                ansiq_core::Constraint::Length(10),
                ansiq_core::Constraint::Length(12),
            ]}
            column_spacing={2u16}
            flex={ansiq_core::Flex::Start}
            block={::ansiq_widgets::Block::bordered().title("Processes")}
            alignments={vec![ansiq_core::TableAlignment::Left, ansiq_core::TableAlignment::Right]}
            selected={Some(1usize)}
        />
    };

    match &tree.kind {
        ansiq_core::ElementKind::Table(props) => {
            assert_eq!(
                props.header.as_ref().unwrap(),
                &ansiq_core::Row::new(["Name", "Status"])
            );
            assert_eq!(
                props.footer.as_ref().unwrap(),
                &ansiq_core::Row::new(["Summary", "2 rows"])
            );
            assert_eq!(props.rows.len(), 2);
            assert_eq!(
                props.widths,
                vec![
                    ansiq_core::Constraint::Length(10),
                    ansiq_core::Constraint::Length(12),
                ]
            );
            assert_eq!(props.column_spacing, 2);
            assert_eq!(props.flex, ansiq_core::Flex::Start);
            assert!(props.block.is_some());
            assert_eq!(
                props.alignments,
                vec![
                    ansiq_core::TableAlignment::Left,
                    ansiq_core::TableAlignment::Right
                ]
            );
            assert_eq!(props.state.selected(), Some(1));
        }
        other => panic!("expected table, got {other:?}"),
    }
}

#[test]
fn view_macro_supports_selection_handlers_for_interactive_primitives() {
    let list: Element<usize> = view! {
        <List
            items={vec!["src".to_string(), "Cargo.toml".to_string()]}
            selected={Some(0usize)}
            on_select={|index| Some(index)}
        />
    };
    let tabs: Element<usize> = view! {
        <Tabs
            titles={vec!["Overview".to_string(), "Files".to_string()]}
            selected={Some(0usize)}
            on_select={|index| Some(index)}
        />
    };
    let table: Element<usize> = view! {
        <Table
            headers={vec!["Name".to_string(), "Status".to_string()]}
            rows={vec![vec!["ansiq".to_string(), "ready".to_string()]]}
            on_select={|index| Some(index)}
        />
    };

    assert!(list.focusable);
    assert!(tabs.focusable);
    assert!(table.focusable);
}

#[test]
fn view_macro_supports_block_wrapped_lists_and_tabs() {
    let tree: Element<()> = view! {
        <Box direction="column">
            <List
                items={vec!["src".to_string(), "Cargo.toml".to_string()]}
                block={::ansiq_widgets::Block::bordered().title("List")}
            />
            <Tabs
                titles={vec!["CPU".to_string(), "Memory".to_string()]}
                block={::ansiq_widgets::Block::bordered().title("Tabs")}
            />
        </Box>
    };

    match &tree.children[0].kind {
        ansiq_core::ElementKind::List(props) => assert!(props.block.is_some()),
        other => panic!("expected list, got {other:?}"),
    }

    match &tree.children[1].kind {
        ansiq_core::ElementKind::Tabs(props) => assert!(props.block.is_some()),
        other => panic!("expected tabs, got {other:?}"),
    }
}

#[test]
fn view_macro_supports_scrollbar_primitive() {
    let tree: Element<usize> = view! {
        <Scrollbar
            orientation={ansiq_core::ScrollbarOrientation::HorizontalBottom}
            position={6usize}
            content_length={12usize}
            viewport_length={3usize}
            symbols={ansiq_core::symbols::scrollbar::HORIZONTAL}
            begin_symbol={None::<String>}
            on_scroll={|position| Some(position)}
        />
    };

    match &tree.kind {
        ansiq_core::ElementKind::Scrollbar(props) => {
            assert_eq!(
                props.orientation,
                ansiq_core::ScrollbarOrientation::HorizontalBottom
            );
            assert_eq!(
                props.state,
                ansiq_core::ScrollbarState::new(12)
                    .position(6)
                    .viewport_content_length(3)
            );
            assert_eq!(props.begin_symbol, None);
            assert_eq!(props.end_symbol.as_deref(), Some("→"));
            assert_eq!(props.track_symbol.as_deref(), Some("─"));
            assert_eq!(props.thumb_symbol, "█");
            assert!(props.on_scroll.as_ref().is_some());
        }
        other => panic!("expected scrollbar, got {other:?}"),
    }

    assert!(tree.focusable);
}

#[test]
fn view_macro_supports_scroll_view_offset_and_scroll_handler() {
    let tree: Element<usize> = view! {
        <ScrollView follow_bottom={true} offset={2usize} on_scroll={|position| Some(position)}>
            <StreamingText content="streaming" />
        </ScrollView>
    };

    assert!(tree.focusable);
    match &tree.kind {
        ansiq_core::ElementKind::ScrollView(props) => {
            assert!(props.follow_bottom);
            assert_eq!(props.offset, Some(2));
            assert!(props.on_scroll.as_ref().is_some());
        }
        other => panic!("expected scroll view, got {other:?}"),
    }
}

#[test]
fn view_macro_supports_clear_and_line_gauge_primitives() {
    let tree: Element<()> = view! {
        <Box direction="column">
            <Clear />
            <LineGauge
                ratio={0.58f64}
                label={ansiq_core::Line::from("58%")}
                line_set={ansiq_core::symbols::line::DOUBLE}
                block={::ansiq_widgets::Block::bordered().title("Line")}
            />
        </Box>
    };

    match &tree.children[0].kind {
        ansiq_core::ElementKind::Clear(_) => {}
        other => panic!("expected clear, got {other:?}"),
    }

    match &tree.children[1].kind {
        ansiq_core::ElementKind::LineGauge(props) => {
            assert!(props.block.is_some());
            assert_eq!(props.ratio, 0.58);
            assert_eq!(props.label, Some(ansiq_core::Line::from("58%")));
            assert_eq!(props.filled_symbol, "═");
            assert_eq!(props.unfilled_symbol, "═");
        }
        other => panic!("expected line gauge, got {other:?}"),
    }
}

#[test]
fn view_macro_supports_sparkline_primitive() {
    let tree: Element<()> = view! {
        <Sparkline
            values={vec![1u64, 2, 4, 6, 8]}
            max={8u64}
            direction={ansiq_core::SparklineDirection::LeftToRight}
            absent_value_symbol={'·'}
            absent_value_style={ansiq_core::Style::default().fg(ansiq_core::Color::DarkGrey)}
        />
    };

    match &tree.kind {
        ansiq_core::ElementKind::Sparkline(props) => {
            assert_eq!(
                props.values,
                vec![Some(1), Some(2), Some(4), Some(6), Some(8)]
            );
            assert_eq!(props.max, Some(8));
        }
        other => panic!("expected sparkline, got {other:?}"),
    }
}

#[test]
fn view_macro_supports_rich_text_primitive() {
    let element: Element<()> = view! {
        <RichText
            block={ansiq_core::HistoryBlock {
                lines: vec![ansiq_core::HistoryLine {
                    runs: vec![ansiq_core::HistoryRun {
                        text: "assistant  planning".to_string(),
                        style: ansiq_core::Style::default(),
                    }],
                }],
            }}
        />
    };

    match &element.kind {
        ElementKind::RichText(props) => {
            assert_eq!(props.block.lines.len(), 1);
            assert_eq!(props.block.lines[0].runs[0].text, "assistant  planning");
        }
        other => panic!("expected rich text, got {other:?}"),
    }
}

#[test]
fn view_macro_supports_barchart_chart_canvas_and_monthly_primitives() {
    let tree: Element<()> = view! {
        <Box direction="column">
            <BarChart bars={vec![("cpu".to_string(), 80u64), ("mem".to_string(), 40u64)]} max={100u64} bar_width={4u16} />
            <Chart datasets={vec![ansiq_core::ChartDataset {
                label: Some("cpu".to_string()),
                points: vec![(0i64, 1i64), (1, 3), (2, 2)],
            }]} min_y={0i64} max_y={4i64} />
            <Canvas width={10u16} height={4u16} cells={vec![ansiq_core::CanvasCell {
                x: 1,
                y: 1,
                symbol: '•',
                style: ansiq_core::Style::default(),
            }]} />
            <Monthly year={2026i32} month={4u8} selected_day={4u8} />
        </Box>
    };

    assert!(matches!(tree.children[0].kind, ElementKind::BarChart(_)));
    assert!(matches!(tree.children[1].kind, ElementKind::Chart(_)));
    assert!(matches!(tree.children[2].kind, ElementKind::Canvas(_)));
    assert!(matches!(tree.children[3].kind, ElementKind::Monthly(_)));
}

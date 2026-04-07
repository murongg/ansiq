use ansiq_core::{
    Alignment, Bar, BarChartProps, BlockProps, BlockTitle, BorderType, Borders, BoxProps,
    CanvasCell, CanvasProps, ChartDataset, ChartProps, ClearProps, Constraint, Direction, Element,
    ElementKind, GaugeProps, HistoryBlock, HistoryLine, HistoryRun, Layout, Length, Line,
    LineGaugeProps, ListProps, ListState, MonthlyProps, Padding, PaneProps, ParagraphProps, Rect,
    Row, ScrollbarProps, ScrollbarState, ShellProps, Span, SparklineProps, StreamingTextProps,
    TableAlignment, TableProps, TableState, TabsProps, TextProps, TitlePosition, Wrap,
};
use ansiq_layout::{layout_tree, measure_height, relayout_tree_along_paths};

fn text(content: &str) -> Element<()> {
    Element::new(ElementKind::Text(TextProps {
        content: content.to_string(),
    }))
}

fn bordered_block_props(title: &str) -> BlockProps {
    BlockProps {
        titles: vec![BlockTitle::new(title)],
        title_alignment: Alignment::Left,
        title_position: TitlePosition::Top,
        borders: Borders::ALL,
        border_type: BorderType::Plain,
        border_set: None,
        padding: Padding::all(1),
        border_style: Default::default(),
        title_style: Default::default(),
    }
}

#[test]
fn column_layout_allocates_fixed_and_fill_children() {
    let root = Element::new(ElementKind::Box(BoxProps {
        direction: Direction::Column,
        gap: 1,
    }))
    .with_children(vec![
        text("status").with_layout(Layout {
            width: Length::Fill,
            height: Length::Fixed(1),
        }),
        text("body").with_layout(Layout {
            width: Length::Fill,
            height: Length::Fill,
        }),
        text("input").with_layout(Layout {
            width: Length::Fill,
            height: Length::Fixed(2),
        }),
    ]);

    let node = layout_tree(root, Rect::new(0, 0, 20, 10));

    assert_eq!(node.rect, Rect::new(0, 0, 20, 10));
    assert_eq!(node.children[0].rect, Rect::new(0, 0, 20, 1));
    assert_eq!(node.children[1].rect, Rect::new(0, 2, 20, 5));
    assert_eq!(node.children[2].rect, Rect::new(0, 8, 20, 2));
}

#[test]
fn row_layout_distributes_remaining_width_without_overflow() {
    let root = Element::new(ElementKind::Box(BoxProps {
        direction: Direction::Row,
        gap: 1,
    }))
    .with_children(vec![
        text("fixed").with_layout(Layout {
            width: Length::Fixed(2),
            height: Length::Fill,
        }),
        text("fill-a").with_layout(Layout {
            width: Length::Fill,
            height: Length::Fill,
        }),
        text("fill-b").with_layout(Layout {
            width: Length::Fill,
            height: Length::Fill,
        }),
    ]);

    let node = layout_tree(root, Rect::new(0, 0, 9, 3));

    assert_eq!(node.children[0].rect, Rect::new(0, 0, 2, 3));
    assert_eq!(node.children[1].rect, Rect::new(3, 0, 3, 3));
    assert_eq!(node.children[2].rect, Rect::new(7, 0, 2, 3));
    assert_eq!(node.children[2].rect.right(), 9);
}

#[test]
fn row_layout_uses_auto_child_widths_instead_of_collapsing_them_to_one_column() {
    let root = Element::new(ElementKind::Box(BoxProps {
        direction: Direction::Row,
        gap: 1,
    }))
    .with_children(vec![
        text("wide").with_layout(Layout {
            width: Length::Auto,
            height: Length::Fill,
        }),
        text("xx").with_layout(Layout {
            width: Length::Auto,
            height: Length::Fill,
        }),
        text("tail").with_layout(Layout {
            width: Length::Auto,
            height: Length::Fill,
        }),
    ]);

    let node = layout_tree(root, Rect::new(0, 0, 20, 3));

    assert_eq!(node.children[0].rect, Rect::new(0, 0, 4, 3));
    assert_eq!(node.children[1].rect, Rect::new(5, 0, 2, 3));
    assert_eq!(node.children[2].rect, Rect::new(8, 0, 4, 3));
}

#[test]
fn pane_insets_child_rect_inside_border() {
    let root = Element::new(ElementKind::Pane(PaneProps {
        title: Some("Output".to_string()),
    }))
    .with_children(vec![text("stream")]);

    let node = layout_tree(root, Rect::new(0, 0, 10, 5));

    assert_eq!(node.rect, Rect::new(0, 0, 10, 5));
    assert_eq!(node.children[0].rect, Rect::new(1, 1, 8, 3));
}

#[test]
fn auto_children_stop_at_remaining_space() {
    let root = Element::new(ElementKind::Box(BoxProps {
        direction: Direction::Column,
        gap: 0,
    }))
    .with_children(vec![text("one"), text("two"), text("three")]);

    let node = layout_tree(root, Rect::new(0, 0, 5, 2));

    assert_eq!(node.children[0].rect, Rect::new(0, 0, 5, 1));
    assert_eq!(node.children[1].rect, Rect::new(0, 1, 5, 1));
    assert_eq!(node.children[2].rect, Rect::new(0, 2, 5, 0));
}

#[test]
fn streaming_text_auto_height_grows_with_wrapped_content() {
    let root: Element<()> = Element::new(ElementKind::Box(BoxProps {
        direction: Direction::Column,
        gap: 0,
    }))
    .with_children(vec![Element::new(ElementKind::StreamingText(
        StreamingTextProps {
            content: "one\ntwo three four".to_string(),
        },
    ))]);

    let node = layout_tree(root, Rect::new(0, 0, 5, 6));

    assert_eq!(node.children[0].rect, Rect::new(0, 0, 5, 4));
}

#[test]
fn rich_text_auto_height_matches_line_count() {
    let root: Element<()> = Element::new(ElementKind::RichText(ansiq_core::RichTextProps {
        block: HistoryBlock {
            lines: vec![
                HistoryLine {
                    runs: vec![HistoryRun {
                        text: "you  123".to_string(),
                        style: Default::default(),
                    }],
                },
                HistoryLine {
                    runs: vec![HistoryRun {
                        text: "assistant  Planning".to_string(),
                        style: Default::default(),
                    }],
                },
                HistoryLine {
                    runs: vec![HistoryRun {
                        text: "Streaming complete.".to_string(),
                        style: Default::default(),
                    }],
                },
            ],
        },
    }));

    let node = layout_tree(root, Rect::new(0, 0, 24, 8));
    assert_eq!(node.measured_height, 3);
}

#[test]
fn measure_height_counts_stacked_transcript_and_input() {
    let root: Element<()> = Element::new(ElementKind::Box(BoxProps {
        direction: Direction::Column,
        gap: 1,
    }))
    .with_children(vec![
        text("status").with_layout(Layout {
            width: Length::Fill,
            height: Length::Fixed(1),
        }),
        Element::new(ElementKind::StreamingText(StreamingTextProps {
            content: "line one\nline two\nline three\nline four".to_string(),
        }))
        .with_layout(Layout {
            width: Length::Fill,
            height: Length::Auto,
        }),
        text("prompt").with_layout(Layout {
            width: Length::Fill,
            height: Length::Fixed(3),
        }),
    ]);

    assert_eq!(measure_height(&root, 20), 10);
}

#[test]
fn layout_tree_records_measured_height_on_each_node() {
    let root = Element::new(ElementKind::Box(BoxProps {
        direction: Direction::Column,
        gap: 1,
    }))
    .with_children(vec![
        text("status"),
        Element::new(ElementKind::Pane(PaneProps {
            title: Some("Output".to_string()),
        }))
        .with_children(vec![text("stream line one\nstream line two")]),
    ]);

    let node = layout_tree(root, Rect::new(0, 0, 12, 10));

    assert_eq!(node.measured_height, 8);
    assert_eq!(node.children[0].measured_height, 1);
    assert_eq!(node.children[1].measured_height, 6);
    assert_eq!(node.children[1].children[0].measured_height, 4);
}

#[test]
fn relayout_along_dirty_paths_only_remeasures_the_dirty_ancestor_chain() {
    let root = Element::new(ElementKind::Box(BoxProps {
        direction: Direction::Column,
        gap: 1,
    }))
    .with_children(vec![
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![text("first")]),
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![
            text("stable alpha"),
            text("stable beta"),
            text("stable gamma"),
        ]),
    ]);

    let bounds = Rect::new(0, 0, 12, 20);
    let mut tree = layout_tree(root, bounds);
    assert_eq!(tree.measured_height, 5);
    assert_eq!(tree.children[1].rect.y, 2);

    let expanded = Element::new(ElementKind::Box(BoxProps {
        direction: Direction::Column,
        gap: 0,
    }))
    .with_children(vec![text("first"), text("second")]);

    tree.children[0] = layout_tree(expanded, tree.children[0].rect);

    let stats = relayout_tree_along_paths(&mut tree, bounds, &[vec![0]]);

    assert_eq!(stats.remeasured_nodes, 2);
    assert!(stats.repositioned_nodes >= 2);
    assert_eq!(stats.invalidated_regions, vec![Rect::new(0, 0, 12, 6)]);
    assert_eq!(tree.measured_height, 6);
    assert_eq!(tree.children[1].rect.y, 3);
}

#[test]
fn relayout_normalizes_overlapping_dirty_paths_to_the_parent_path() {
    let bounds = Rect::new(0, 0, 12, 20);

    let build_root = || {
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 1,
        }))
        .with_children(vec![
            Element::new(ElementKind::Box(BoxProps {
                direction: Direction::Column,
                gap: 0,
            }))
            .with_children(vec![
                Element::new(ElementKind::Box(BoxProps {
                    direction: Direction::Column,
                    gap: 0,
                }))
                .with_children(vec![text("alpha")]),
            ]),
            Element::new(ElementKind::Box(BoxProps {
                direction: Direction::Column,
                gap: 0,
            }))
            .with_children(vec![text("stable")]),
        ])
    };
    let build_expanded = || {
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![
            Element::new(ElementKind::Box(BoxProps {
                direction: Direction::Column,
                gap: 0,
            }))
            .with_children(vec![text("alpha"), text("beta")]),
        ])
    };

    let mut parent_tree = layout_tree(build_root(), bounds);
    parent_tree.children[0] = layout_tree(build_expanded(), parent_tree.children[0].rect);

    let mut overlapping_tree = layout_tree(build_root(), bounds);
    overlapping_tree.children[0] = layout_tree(build_expanded(), overlapping_tree.children[0].rect);

    let parent_only = relayout_tree_along_paths(&mut parent_tree, bounds, &[vec![0]]);
    let overlapping =
        relayout_tree_along_paths(&mut overlapping_tree, bounds, &[vec![0], vec![0, 0]]);

    assert_eq!(overlapping.remeasured_nodes, parent_only.remeasured_nodes);
    assert_eq!(
        overlapping.repositioned_nodes,
        parent_only.repositioned_nodes
    );
    assert_eq!(
        overlapping.invalidated_regions,
        parent_only.invalidated_regions
    );
}

#[test]
fn relayout_does_not_invalidate_entire_fill_container_when_only_children_shift() {
    let root = Element::new(ElementKind::Box(BoxProps {
        direction: Direction::Column,
        gap: 0,
    }))
    .with_children(vec![
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_children(vec![text("header")]),
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_layout(Layout {
            width: Length::Fill,
            height: Length::Fill,
        })
        .with_children(vec![text("body")]),
    ]);

    let bounds = Rect::new(0, 0, 12, 10);
    let mut tree = layout_tree(root, bounds);
    assert_eq!(tree.children[1].rect, Rect::new(0, 1, 12, 9));
    assert_eq!(tree.children[1].children[0].rect, Rect::new(0, 1, 12, 1));

    let expanded = Element::new(ElementKind::Box(BoxProps {
        direction: Direction::Column,
        gap: 0,
    }))
    .with_children(vec![text("header"), text("details")]);

    tree.children[0] = layout_tree(expanded, tree.children[0].rect);

    let stats = relayout_tree_along_paths(&mut tree, bounds, &[vec![0]]);

    assert_eq!(tree.children[1].rect, Rect::new(0, 2, 12, 8));
    assert_eq!(tree.children[1].children[0].rect, Rect::new(0, 2, 12, 1));
    assert!(
        !stats.invalidated_regions.contains(&Rect::new(0, 1, 12, 9)),
        "layout-only fill container should not invalidate its entire old rect"
    );
    assert!(
        !stats.invalidated_regions.contains(&Rect::new(0, 2, 12, 8)),
        "layout-only fill container should not invalidate its entire new rect"
    );
    assert_eq!(stats.invalidated_regions, vec![Rect::new(0, 0, 12, 3)]);
}

#[test]
fn block_insets_children_like_a_bordered_container() {
    let root: Element<()> = Element::new(ElementKind::Block(BlockProps {
        titles: vec![BlockTitle::new("Inspector")],
        title_alignment: Alignment::Left,
        title_position: TitlePosition::Top,
        borders: Borders::ALL,
        border_type: BorderType::Plain,
        border_set: None,
        padding: Padding::all(1),
        border_style: Default::default(),
        title_style: Default::default(),
    }))
    .with_children(vec![Element::new(ElementKind::Paragraph(ParagraphProps {
        content: "line one\nline two".into(),
        block: None,
        alignment: Alignment::Left,
        wrap: Some(Wrap { trim: false }),
        scroll_x: 0,
        scroll_y: 0,
    }))]);

    let node = layout_tree(root, Rect::new(0, 0, 14, 6));

    assert_eq!(node.rect, Rect::new(0, 0, 14, 6));
    assert_eq!(node.children[0].rect, Rect::new(2, 2, 10, 2));
    assert_eq!(node.measured_height, 6);
}

#[test]
fn block_titles_reserve_vertical_space_even_without_matching_borders() {
    let root: Element<()> = Element::new(ElementKind::Block(BlockProps {
        titles: vec![BlockTitle::top("Top"), BlockTitle::bottom("Bottom")],
        title_alignment: Alignment::Left,
        title_position: TitlePosition::Top,
        borders: Borders::LEFT | Borders::RIGHT,
        border_type: BorderType::Plain,
        border_set: None,
        padding: Padding::zero(),
        border_style: Default::default(),
        title_style: Default::default(),
    }))
    .with_children(vec![text("body")]);

    let node = layout_tree(root, Rect::new(0, 0, 10, 5));

    assert_eq!(node.children[0].rect, Rect::new(1, 1, 8, 1));
}

#[test]
fn paragraph_block_adds_border_and_padding_to_measured_height() {
    let paragraph: Element<()> = Element::new(ElementKind::Paragraph(ParagraphProps {
        content: "line one\nline two".into(),
        block: Some(ansiq_core::BlockFrame {
            props: bordered_block_props("Paragraph"),
            style: Default::default(),
        }),
        alignment: Alignment::Left,
        wrap: Some(Wrap { trim: false }),
        scroll_x: 0,
        scroll_y: 0,
    }));

    let node = layout_tree(paragraph, Rect::new(0, 0, 16, 10));

    assert_eq!(node.measured_height, 6);
}

#[test]
fn list_auto_height_matches_the_number_of_items() {
    let list: Element<()> = Element::new(ElementKind::List(ListProps {
        block: None,
        items: vec!["src".into(), "Cargo.toml".into(), "README.md".into()],
        state: ListState::default().with_selected(Some(1)),
        highlight_symbol: Some(Line::from(">>")),
        highlight_style: Default::default(),
        highlight_spacing: ansiq_core::HighlightSpacing::WhenSelected,
        repeat_highlight_symbol: false,
        direction: ansiq_core::ListDirection::TopToBottom,
        scroll_padding: 0,
        on_select: None,
    }));

    let node = layout_tree(list, Rect::new(0, 0, 20, 8));

    assert_eq!(node.rect, Rect::new(0, 0, 20, 8));
    assert_eq!(node.measured_height, 3);
}

#[test]
fn list_block_adds_border_and_padding_to_measured_height() {
    let list: Element<()> = Element::new(ElementKind::List(ListProps {
        block: Some(ansiq_core::BlockFrame {
            props: bordered_block_props("List"),
            style: Default::default(),
        }),
        items: vec!["src".into(), "Cargo.toml".into()],
        state: ListState::default(),
        highlight_symbol: None,
        highlight_style: Default::default(),
        highlight_spacing: ansiq_core::HighlightSpacing::WhenSelected,
        repeat_highlight_symbol: false,
        direction: ansiq_core::ListDirection::TopToBottom,
        scroll_padding: 0,
        on_select: None,
    }));

    let node = layout_tree(list, Rect::new(0, 0, 20, 8));
    assert_eq!(node.measured_height, 6);
}

#[test]
fn tabs_and_gauge_use_single_line_auto_heights() {
    let tabs: Element<()> = Element::new(ElementKind::Tabs(TabsProps {
        block: None,
        titles: vec!["Overview".into(), "Files".into(), "Logs".into()],
        selected: Some(1),
        selection_explicit: false,
        highlight_style: Default::default(),
        divider: Span::raw("|"),
        padding_left: Line::from(" "),
        padding_right: Line::from(" "),
        on_select: None,
    }));
    let gauge: Element<()> = Element::new(ElementKind::Gauge(GaugeProps {
        block: None,
        ratio: 0.72,
        label: Some(Span::raw("72%")),
        use_unicode: false,
        gauge_style: ansiq_core::Style::default(),
    }));

    let tabs_node = layout_tree(tabs, Rect::new(0, 0, 24, 4));
    let gauge_node = layout_tree(gauge, Rect::new(0, 0, 24, 4));

    assert_eq!(tabs_node.measured_height, 1);
    assert_eq!(gauge_node.measured_height, 1);
}

#[test]
fn gauge_and_line_gauge_blocks_add_border_and_padding_to_measured_height() {
    let gauge: Element<()> = Element::new(ElementKind::Gauge(GaugeProps {
        block: Some(ansiq_core::BlockFrame {
            props: bordered_block_props("Gauge"),
            style: Default::default(),
        }),
        ratio: 0.72,
        label: Some(Span::raw("72%")),
        use_unicode: false,
        gauge_style: ansiq_core::Style::default(),
    }));
    let line_gauge: Element<()> = Element::new(ElementKind::LineGauge(LineGaugeProps {
        block: Some(ansiq_core::BlockFrame {
            props: bordered_block_props("Line"),
            style: Default::default(),
        }),
        ratio: 0.58,
        label: Some(Line::from("58%")),
        filled_symbol: "━".to_string(),
        unfilled_symbol: "─".to_string(),
        filled_style: ansiq_core::Style::default(),
        unfilled_style: ansiq_core::Style::default(),
    }));

    let gauge_node = layout_tree(gauge, Rect::new(0, 0, 24, 8));
    let line_node = layout_tree(line_gauge, Rect::new(0, 0, 24, 8));

    assert_eq!(gauge_node.measured_height, 5);
    assert_eq!(line_node.measured_height, 5);
}

#[test]
fn tabs_block_adds_border_and_padding_to_measured_height() {
    let tabs: Element<()> = Element::new(ElementKind::Tabs(TabsProps {
        block: Some(ansiq_core::BlockFrame {
            props: bordered_block_props("Tabs"),
            style: Default::default(),
        }),
        titles: vec!["Overview".into(), "Files".into()],
        selected: Some(0),
        selection_explicit: false,
        highlight_style: Default::default(),
        divider: Span::raw("|"),
        padding_left: Line::from(" "),
        padding_right: Line::from(" "),
        on_select: None,
    }));

    let node = layout_tree(tabs, Rect::new(0, 0, 24, 6));
    assert_eq!(node.measured_height, 5);
}

#[test]
fn shell_keeps_header_and_footer_visible_when_body_overflows() {
    let shell: Element<()> = Element::new(ElementKind::Shell(ShellProps)).with_children(vec![
        text("status").with_layout(Layout {
            width: Length::Fill,
            height: Length::Auto,
        }),
        Element::new(ElementKind::StreamingText(StreamingTextProps {
            content: "line 1\nline 2\nline 3\nline 4\nline 5\nline 6".to_string(),
        }))
        .with_layout(Layout {
            width: Length::Fill,
            height: Length::Auto,
        }),
        text("composer").with_layout(Layout {
            width: Length::Fill,
            height: Length::Fixed(3),
        }),
    ]);

    let node = layout_tree(shell, Rect::new(0, 0, 20, 5));

    assert_eq!(node.children[0].rect, Rect::new(0, 0, 20, 1));
    assert_eq!(node.children[2].rect, Rect::new(0, 2, 20, 3));
    assert_eq!(node.children[1].rect, Rect::new(0, 1, 20, 1));
    assert_eq!(node.measured_height, 10);
}

#[test]
fn table_auto_height_counts_header_and_rows() {
    let table: Element<()> = Element::new(ElementKind::Table(TableProps {
        block: Some(ansiq_core::BlockFrame {
            props: bordered_block_props("Processes"),
            style: Default::default(),
        }),
        header: Some(Row::new(["Name", "Status"])),
        footer: Some(Row::new(["Summary", "2 rows"])),
        rows: vec![
            Row::new(["ansiq", "ready"]),
            Row::new(["codexdemo", "streaming"]),
        ],
        widths: vec![Constraint::Length(10), Constraint::Length(12)],
        column_spacing: 2,
        flex: ansiq_core::Flex::Start,
        alignments: vec![TableAlignment::Left, TableAlignment::Right],
        state: TableState::default().with_selected(Some(1)),
        highlight_symbol: None,
        row_highlight_style: Default::default(),
        column_highlight_style: Default::default(),
        cell_highlight_style: Default::default(),
        highlight_spacing: ansiq_core::HighlightSpacing::WhenSelected,
        on_select: None,
    }));

    let node = layout_tree(table, Rect::new(0, 0, 24, 8));

    assert_eq!(node.measured_height, 8);
}

#[test]
fn scrollbar_uses_a_fixed_track_width_and_parent_height() {
    let root: Element<()> = Element::new(ElementKind::Box(BoxProps {
        direction: Direction::Row,
        gap: 1,
    }))
    .with_children(vec![
        text("output").with_layout(Layout {
            width: Length::Fill,
            height: Length::Fill,
        }),
        Element::new(ElementKind::Scrollbar(ScrollbarProps {
            state: ScrollbarState::new(12)
                .position(6)
                .viewport_content_length(3),
            orientation: ansiq_core::ScrollbarOrientation::VerticalRight,
            thumb_symbol: "█".to_string(),
            thumb_style: ansiq_core::Style::default(),
            track_symbol: Some("░".to_string()),
            track_style: ansiq_core::Style::default(),
            begin_symbol: Some("↑".to_string()),
            begin_style: ansiq_core::Style::default(),
            end_symbol: Some("↓".to_string()),
            end_style: ansiq_core::Style::default(),
            on_scroll: None,
        }))
        .with_layout(Layout {
            width: Length::Fixed(1),
            height: Length::Fill,
        }),
    ]);

    let node = layout_tree(root, Rect::new(0, 0, 12, 6));

    assert_eq!(node.children[1].rect, Rect::new(11, 0, 1, 6));
}

#[test]
fn clear_fills_its_parent_and_line_gauge_uses_single_line_height() {
    let root: Element<()> = Element::new(ElementKind::Box(BoxProps {
        direction: Direction::Column,
        gap: 1,
    }))
    .with_children(vec![
        Element::new(ElementKind::Clear(ClearProps)).with_layout(Layout {
            width: Length::Fill,
            height: Length::Fill,
        }),
        Element::new(ElementKind::LineGauge(LineGaugeProps {
            block: None,
            ratio: 0.58,
            label: Some(Line::from("58%")),
            filled_symbol: "━".to_string(),
            unfilled_symbol: "─".to_string(),
            filled_style: ansiq_core::Style::default(),
            unfilled_style: ansiq_core::Style::default(),
        })),
    ]);

    let node = layout_tree(root, Rect::new(0, 0, 12, 6));

    assert_eq!(node.children[0].rect, Rect::new(0, 0, 12, 4));
    assert_eq!(node.children[1].measured_height, 1);
}

#[test]
fn sparkline_uses_single_line_auto_height() {
    let sparkline: Element<()> = Element::new(ElementKind::Sparkline(SparklineProps {
        values: vec![Some(1), Some(2), Some(4), Some(6), Some(8)],
        max: Some(8),
        direction: ansiq_core::SparklineDirection::LeftToRight,
        absent_value_symbol: '·',
        absent_value_style: ansiq_core::Style::default(),
    }));

    let node = layout_tree(sparkline, Rect::new(0, 0, 12, 4));

    assert_eq!(node.measured_height, 1);
}

#[test]
fn barchart_chart_canvas_and_monthly_use_stable_default_heights() {
    let barchart: Element<()> = Element::new(ElementKind::BarChart(BarChartProps {
        bars: vec![
            Bar {
                label: "cpu".to_string(),
                value: 64,
            },
            Bar {
                label: "mem".to_string(),
                value: 42,
            },
        ],
        max: Some(100),
        bar_width: 3,
    }));
    let chart: Element<()> = Element::new(ElementKind::Chart(ChartProps {
        datasets: vec![ChartDataset {
            label: Some("cpu".to_string()),
            points: vec![(0, 1), (1, 3), (2, 2)],
        }],
        min_y: Some(0),
        max_y: Some(4),
    }));
    let canvas: Element<()> = Element::new(ElementKind::Canvas(CanvasProps {
        width: 10,
        height: 4,
        cells: vec![CanvasCell {
            x: 1,
            y: 1,
            symbol: '•',
            style: Default::default(),
        }],
    }));
    let monthly: Element<()> = Element::new(ElementKind::Monthly(MonthlyProps {
        year: 2026,
        month: 4,
        selected_day: Some(4),
    }));

    let barchart = layout_tree(barchart, Rect::new(0, 0, 24, 8));
    let chart = layout_tree(chart, Rect::new(0, 0, 24, 10));
    let canvas = layout_tree(canvas, Rect::new(0, 0, 24, 10));
    let monthly = layout_tree(monthly, Rect::new(0, 0, 24, 10));

    assert_eq!(barchart.measured_height, 6);
    assert_eq!(chart.measured_height, 8);
    assert_eq!(canvas.measured_height, 8);
    assert_eq!(monthly.measured_height, 8);
}

#[test]
#[should_panic(expected = "Shell takes at most 3 children: header / body / footer")]
fn shell_panics_in_debug_when_given_more_than_three_children() {
    let shell: Element<()> = Element::new(ElementKind::Shell(ShellProps)).with_children(vec![
        text("header"),
        text("body"),
        text("footer"),
        text("extra"),
    ]);

    let _ = layout_tree(shell, Rect::new(0, 0, 20, 10));
}

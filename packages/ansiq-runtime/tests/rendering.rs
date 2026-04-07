use ansiq_core::{
    Alignment, Bar, BarChartProps, BlockProps, BlockTitle, BorderType, Borders, BoxProps,
    CanvasCell, CanvasProps, ChartDataset, ChartProps, ClearProps, Color, Constraint, Direction,
    Element, ElementKind, GaugeProps, HighlightSpacing, HistoryBlock, HistoryLine, HistoryRun,
    InputProps, Layout, Length, Line, LineGaugeProps, ListDirection, ListItem, ListProps,
    ListState, MonthlyProps, Node, Padding, PaneProps, ParagraphProps, Rect, Row, ScrollViewProps,
    ScrollbarProps, ScrollbarState, Span, SparklineProps, StatusBarProps, StreamingTextProps,
    Style, TableAlignment, TableProps, TableState, TabsProps, Text, TitlePosition, Wrap,
};
use ansiq_layout::layout_tree;
use ansiq_render::FrameBuffer;
use ansiq_runtime::{cursor_position, draw_tree, draw_tree_in_regions};

fn cell_string(buffer: &FrameBuffer, y: u16) -> String {
    (0..buffer.width())
        .map(|x| buffer.get(x, y).symbol)
        .collect::<String>()
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
fn draw_tree_prefers_ink_like_minimal_console_styles() {
    let tree = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 0,
        }))
        .with_style(Style::default())
        .with_children(vec![
            Element::new(ElementKind::StatusBar(StatusBarProps {
                content: " ansiq · runtime-first tui · idle ".to_string(),
            }))
            .with_layout(Layout {
                width: Length::Fill,
                height: Length::Fixed(1),
            })
            .with_style(Style::default().fg(Color::Cyan)),
            Element::new(ElementKind::Pane(PaneProps {
                title: Some("Session".to_string()),
            }))
            .with_layout(Layout {
                width: Length::Fill,
                height: Length::Fill,
            })
            .with_style(Style::default().fg(Color::Grey)),
            (Element::new(ElementKind::Input(InputProps::<()> {
                value: "hello".to_string(),
                placeholder: "Type".to_string(),
                on_change: None,
                on_submit: None,
                cursor: "hello".chars().count(),
            }))
            .with_layout(Layout {
                width: Length::Fill,
                height: Length::Fixed(3),
            })
            .with_style(Style::default())
            .with_focusable(true)),
        ]),
        Rect::new(0, 0, 20, 8),
    );

    let mut buffer = FrameBuffer::new(20, 8);
    let focused_input = tree.children[2].id;
    draw_tree(&tree, Some(focused_input), &mut buffer);

    assert!(cell_string(&buffer, 0).starts_with(" ansiq"));
    assert_eq!(buffer.get(0, 0).style.bg, Color::Reset);
    assert_eq!(buffer.get(1, 0).style.fg, Color::Cyan);
    assert!(!buffer.get(1, 0).style.bold);

    assert_eq!(buffer.get(0, 1).symbol, '╭');
    assert_eq!(buffer.get(1, 1).symbol, 'S');
    assert!(cell_string(&buffer, 1).contains("Session"));
    assert_eq!(buffer.get(3, 1).style.fg, Color::Grey);
    assert_eq!(buffer.get(0, 5).symbol, '╭');
    assert_eq!(buffer.get(0, 5).style.fg, Color::Grey);
    assert!(!buffer.get(0, 5).style.bold);

    assert!(cell_string(&buffer, 6).contains("hello"));
    assert_eq!(buffer.get(1, 6).style.fg, Color::Reset);
    assert_eq!(buffer.get(1, 6).style.bg, Color::Reset);
}

#[test]
fn scroll_view_follow_bottom_shows_latest_lines() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::ScrollView(ScrollViewProps {
            follow_bottom: true,
            offset: None,
            on_scroll: None,
        }))
        .with_children(vec![Element::new(ElementKind::StreamingText(
            StreamingTextProps {
                content: "one\ntwo\nthree\nfour".to_string(),
            },
        ))]),
        Rect::new(0, 0, 10, 2),
    );

    let mut buffer = FrameBuffer::new(10, 2);
    draw_tree(&tree, None, &mut buffer);

    assert!(cell_string(&buffer, 0).starts_with("three"));
    assert!(cell_string(&buffer, 1).starts_with("four"));
}

#[test]
fn scroll_view_offset_shows_the_requested_slice_from_the_top() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::ScrollView(ScrollViewProps {
            follow_bottom: false,
            offset: Some(1),
            on_scroll: None,
        }))
        .with_children(vec![Element::new(ElementKind::StreamingText(
            StreamingTextProps {
                content: "one\ntwo\nthree\nfour".to_string(),
            },
        ))]),
        Rect::new(0, 0, 10, 2),
    );

    let mut buffer = FrameBuffer::new(10, 2);
    draw_tree(&tree, None, &mut buffer);

    assert!(cell_string(&buffer, 0).starts_with("two"));
    assert!(cell_string(&buffer, 1).starts_with("three"));
}

#[test]
fn input_cursor_tracks_display_width_for_wide_characters() {
    let tree = layout_tree(
        Element::new(ElementKind::Input(InputProps::<()> {
            value: "大河向东流".to_string(),
            placeholder: String::new(),
            on_change: None,
            on_submit: None,
            cursor: "大河向东流".chars().count(),
        }))
        .with_layout(Layout {
            width: Length::Fill,
            height: Length::Fixed(3),
        })
        .with_style(Style::default())
        .with_focusable(true),
        Rect::new(0, 0, 14, 3),
    );

    let mut buffer = FrameBuffer::new(14, 3);
    draw_tree(&tree, Some(tree.id), &mut buffer);

    assert_eq!(buffer.get(1, 1).symbol, '大');
    assert_eq!(buffer.get(3, 1).symbol, '河');
    assert_eq!(cursor_position(&tree, Some(tree.id)), Some((11, 1)));
}

#[test]
fn focused_input_does_not_overwrite_next_wide_character() {
    let tree = layout_tree(
        Element::new(ElementKind::Input(InputProps::<()> {
            value: "你好".to_string(),
            placeholder: String::new(),
            on_change: None,
            on_submit: None,
            cursor: 1,
        }))
        .with_layout(Layout {
            width: Length::Fill,
            height: Length::Fixed(3),
        })
        .with_style(Style::default())
        .with_focusable(true),
        Rect::new(0, 0, 8, 3),
    );

    let mut buffer = FrameBuffer::new(8, 3);
    draw_tree(&tree, Some(tree.id), &mut buffer);

    assert_eq!(buffer.get(1, 1).symbol, '你');
    assert_eq!(buffer.get(3, 1).symbol, '好');
}

#[test]
fn draw_tree_keeps_streaming_text_role_labels_neutral_by_default() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::StreamingText(StreamingTextProps {
            content: "assistant  Planning the change\nuser  write tests".to_string(),
        }))
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 36, 2),
    );

    let mut buffer = FrameBuffer::new(36, 2);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(buffer.get(0, 0).symbol, 'a');
    assert_eq!(buffer.get(0, 0).style.fg, Color::Grey);
    assert_eq!(buffer.get(0, 1).symbol, 'u');
    assert_eq!(buffer.get(0, 1).style.fg, Color::Grey);
}

#[test]
fn rich_text_renders_styled_runs_across_multiple_lines() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::RichText(ansiq_core::RichTextProps {
            block: HistoryBlock {
                lines: vec![
                    HistoryLine {
                        runs: vec![
                            HistoryRun {
                                text: "you  ".to_string(),
                                style: Style::default().fg(Color::White).bold(true),
                            },
                            HistoryRun {
                                text: "123".to_string(),
                                style: Style::default().fg(Color::Grey),
                            },
                        ],
                    },
                    HistoryLine {
                        runs: vec![
                            HistoryRun {
                                text: "assistant  ".to_string(),
                                style: Style::default().fg(Color::Cyan),
                            },
                            HistoryRun {
                                text: "Planning".to_string(),
                                style: Style::default().fg(Color::Grey),
                            },
                        ],
                    },
                ],
            },
        }))
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 24, 4),
    );

    let mut buffer = FrameBuffer::new(24, 4);
    draw_tree(&tree, None, &mut buffer);

    assert!(cell_string(&buffer, 0).starts_with("you  123"));
    assert_eq!(buffer.get(0, 0).style.fg, Color::White);
    assert!(buffer.get(0, 0).style.bold);
    assert_eq!(buffer.get(5, 0).style.fg, Color::Grey);
    assert!(cell_string(&buffer, 1).starts_with("assistant  Planning"));
    assert_eq!(buffer.get(0, 1).style.fg, Color::Cyan);
    assert_eq!(buffer.get(11, 1).style.fg, Color::Grey);
}

#[test]
fn focused_input_border_uses_explicit_component_color() {
    let tree = layout_tree(
        Element::new(ElementKind::Input(InputProps::<()> {
            value: String::new(),
            placeholder: "prompt".to_string(),
            on_change: None,
            on_submit: None,
            cursor: 0,
        }))
        .with_layout(Layout {
            width: Length::Fill,
            height: Length::Fixed(3),
        })
        .with_style(Style::default().fg(Color::White))
        .with_focusable(true),
        Rect::new(0, 0, 16, 3),
    );

    let mut buffer = FrameBuffer::new(16, 3);
    draw_tree(&tree, Some(tree.id), &mut buffer);

    assert_eq!(buffer.get(0, 0).symbol, '╭');
    assert_eq!(buffer.get(0, 0).style.fg, Color::White);
}

#[test]
fn block_and_list_render_bordered_content_and_selection_state() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Block(BlockProps {
            titles: vec![BlockTitle::new("Files")],
            title_alignment: Alignment::Left,
            title_position: TitlePosition::Top,
            borders: Borders::ALL,
            border_type: BorderType::Rounded,
            border_set: None,
            padding: Padding::zero(),
            border_style: Style::default().fg(Color::Blue),
            title_style: Style::default().fg(Color::Yellow),
        }))
        .with_layout(Layout {
            width: Length::Fill,
            height: Length::Fill,
        })
        .with_style(Style::default().fg(Color::Grey))
        .with_children(vec![
            Element::new(ElementKind::Paragraph(ParagraphProps {
                content: "workspace".into(),
                block: None,
                alignment: Alignment::Left,
                wrap: Some(Wrap { trim: false }),
                scroll_x: 0,
                scroll_y: 0,
            })),
            Element::new(ElementKind::List(ListProps {
                block: None,
                items: vec!["src".into(), "Cargo.toml".into()],
                state: ListState::default().with_selected(Some(1)),
                highlight_symbol: Some(Line::from(">>")),
                highlight_style: Style::default(),
                highlight_spacing: HighlightSpacing::WhenSelected,
                repeat_highlight_symbol: false,
                direction: ListDirection::TopToBottom,
                scroll_padding: 0,
                on_select: None,
            })),
        ]),
        Rect::new(0, 0, 18, 6),
    );

    let mut buffer = FrameBuffer::new(18, 6);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(buffer.get(0, 0).symbol, '╭');
    assert!(cell_string(&buffer, 0).contains("Files"));
    assert!(cell_string(&buffer, 1).contains("workspace"));
    assert!(cell_string(&buffer, 2).contains("src"));
    assert!(cell_string(&buffer, 3).contains("Cargo.toml"));
    assert!(cell_string(&buffer, 3).contains(">>"));
    assert!(cell_string(&buffer, 3).contains("Cargo.toml"));
    assert!(buffer.get(1, 3).style.bold);
    assert!(buffer.get(1, 3).style.reversed);
}

#[test]
fn block_can_render_partial_double_borders() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Block(BlockProps {
            titles: vec![BlockTitle::top("Logs")],
            title_alignment: Alignment::Left,
            title_position: TitlePosition::Top,
            borders: Borders::LEFT | Borders::RIGHT,
            border_type: BorderType::Double,
            border_set: None,
            padding: Padding::zero(),
            border_style: Style::default().fg(Color::Cyan),
            title_style: Style::default().fg(Color::Yellow),
        }))
        .with_layout(Layout {
            width: Length::Fill,
            height: Length::Fill,
        })
        .with_children(vec![Element::new(ElementKind::Paragraph(ParagraphProps {
            content: "stream".into(),
            block: None,
            alignment: Alignment::Left,
            wrap: None,
            scroll_x: 0,
            scroll_y: 0,
        }))]),
        Rect::new(0, 0, 12, 3),
    );

    let mut buffer = FrameBuffer::new(12, 3);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(buffer.get(0, 0).symbol, '║');
    assert_eq!(buffer.get(11, 0).symbol, '║');
    assert_eq!(buffer.get(0, 1).symbol, '║');
    assert_eq!(buffer.get(11, 1).symbol, '║');
    assert_eq!(buffer.get(0, 0).style.fg, Color::Cyan);
    assert!(cell_string(&buffer, 0).contains("Logs"));
    assert!(cell_string(&buffer, 1).contains("stream"));
}

#[test]
fn block_can_render_bottom_titles_without_bottom_borders() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Block(BlockProps {
            titles: vec![BlockTitle::bottom(Line::from("Bottom").right_aligned())],
            title_alignment: Alignment::Left,
            title_position: TitlePosition::Top,
            borders: Borders::LEFT | Borders::RIGHT,
            border_type: BorderType::Plain,
            border_set: None,
            padding: Padding::zero(),
            border_style: Style::default().fg(Color::Cyan),
            title_style: Style::default().fg(Color::Yellow),
        }))
        .with_layout(Layout {
            width: Length::Fill,
            height: Length::Fill,
        })
        .with_children(vec![Element::new(ElementKind::Paragraph(ParagraphProps {
            content: "body".into(),
            block: None,
            alignment: Alignment::Left,
            wrap: None,
            scroll_x: 0,
            scroll_y: 0,
        }))]),
        Rect::new(0, 0, 12, 4),
    );

    let mut buffer = FrameBuffer::new(12, 4);
    draw_tree(&tree, None, &mut buffer);

    assert!(cell_string(&buffer, 3).contains("Bottom"));
    assert!(cell_string(&buffer, 0).contains("body"));
}

#[test]
fn block_renders_custom_border_set_like_ratatui() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Block(BlockProps {
            titles: vec![BlockTitle::new("Custom")],
            title_alignment: Alignment::Left,
            title_position: TitlePosition::Top,
            borders: Borders::ALL,
            border_type: BorderType::Plain,
            border_set: Some(ansiq_core::symbols::border::Set {
                top_left: 'A',
                top_right: 'B',
                bottom_left: 'C',
                bottom_right: 'D',
                horizontal_top: '=',
                horizontal_bottom: '_',
                vertical_left: '!',
                vertical_right: '?',
            }),
            padding: Padding::zero(),
            border_style: Style::default(),
            title_style: Style::default(),
        })),
        Rect::new(0, 0, 12, 3),
    );

    let mut buffer = FrameBuffer::new(12, 3);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(buffer.get(0, 0).symbol, 'A');
    assert_eq!(buffer.get(11, 0).symbol, 'B');
    assert_eq!(buffer.get(0, 2).symbol, 'C');
    assert_eq!(buffer.get(11, 2).symbol, 'D');
    assert_eq!(buffer.get(0, 1).symbol, '!');
    assert_eq!(buffer.get(11, 1).symbol, '?');
}

#[test]
fn block_titles_do_not_overlap_when_left_center_and_right_groups_compete() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Block(BlockProps {
            titles: vec![
                BlockTitle::top(Line::from("Left").left_aligned()),
                BlockTitle::top(Line::from("Center").centered()),
                BlockTitle::top(Line::from("Right").right_aligned()),
            ],
            title_alignment: Alignment::Left,
            title_position: TitlePosition::Top,
            borders: Borders::ALL,
            border_type: BorderType::Plain,
            border_set: None,
            padding: Padding::zero(),
            border_style: Style::default(),
            title_style: Style::default(),
        })),
        Rect::new(0, 0, 12, 3),
    );

    let mut buffer = FrameBuffer::new(12, 3);
    draw_tree(&tree, None, &mut buffer);

    let top = cell_string(&buffer, 0);
    assert!(top.contains("Left"));
    assert!(top.contains("Right"));
    assert!(!top.contains("Center"));
}

#[test]
fn list_can_render_inside_an_embedded_block() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::List(ListProps {
            block: Some(ansiq_core::BlockFrame {
                props: bordered_block_props("List"),
                style: Style::default().fg(Color::Grey),
            }),
            items: vec!["src".into(), "Cargo.toml".into()],
            state: ListState::default().with_selected(Some(1)),
            highlight_symbol: Some(Line::from(">>")),
            highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            repeat_highlight_symbol: false,
            direction: ListDirection::TopToBottom,
            scroll_padding: 0,
            on_select: None,
        })),
        Rect::new(0, 0, 18, 6),
    );

    let mut buffer = FrameBuffer::new(18, 6);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(buffer.get(0, 0).symbol, '┌');
    assert!(cell_string(&buffer, 0).contains("List"));
    assert!(cell_string(&buffer, 2).contains("src"));
    assert!(cell_string(&buffer, 3).contains("Cargo.toml"));
}

#[test]
fn list_can_render_bottom_to_top_like_ratatui() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::List(ListProps {
            block: None,
            items: vec!["alpha".into(), "beta".into(), "gamma".into()],
            state: ListState::default(),
            highlight_symbol: Some(Line::from(">>")),
            highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            repeat_highlight_symbol: false,
            direction: ListDirection::BottomToTop,
            scroll_padding: 0,
            on_select: None,
        })),
        Rect::new(0, 0, 10, 3),
    );

    let mut buffer = FrameBuffer::new(10, 3);
    draw_tree(&tree, None, &mut buffer);

    assert!(cell_string(&buffer, 0).starts_with("gamma"));
    assert!(cell_string(&buffer, 1).starts_with("beta"));
    assert!(cell_string(&buffer, 2).starts_with("alpha"));
}

#[test]
fn list_auto_scrolls_selected_item_into_view_without_changing_text_width() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::List(ListProps {
            block: None,
            items: vec![
                "alpha".into(),
                "beta".into(),
                "gamma".into(),
                "delta".into(),
                "epsilon".into(),
            ],
            state: ListState::default().with_selected(Some(4)),
            highlight_symbol: Some(Line::from(">>")),
            highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            repeat_highlight_symbol: false,
            direction: ListDirection::TopToBottom,
            scroll_padding: 0,
            on_select: None,
        }))
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 12, 3),
    );

    let mut buffer = FrameBuffer::new(12, 3);
    draw_tree(&tree, None, &mut buffer);

    assert!(cell_string(&buffer, 0).contains("gamma"));
    assert!(cell_string(&buffer, 1).contains("delta"));
    assert!(cell_string(&buffer, 2).contains(">>"));
    assert!(cell_string(&buffer, 2).contains("epsilon"));
    assert!(buffer.get(0, 2).style.bold);
    assert!(buffer.get(0, 2).style.reversed);
}

#[test]
fn list_renders_multiline_items_and_repeats_the_highlight_symbol() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::List(ListProps {
            block: None,
            items: vec![ListItem::new("alpha\nbeta"), ListItem::new("gamma")],
            state: ListState::default().with_selected(Some(0)),
            highlight_symbol: Some(Line::from(">>")),
            highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::Always,
            repeat_highlight_symbol: true,
            direction: ListDirection::TopToBottom,
            scroll_padding: 0,
            on_select: None,
        }))
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 12, 3),
    );

    let mut buffer = FrameBuffer::new(12, 3);
    draw_tree(&tree, None, &mut buffer);

    assert!(cell_string(&buffer, 0).contains(">>"));
    assert!(cell_string(&buffer, 0).contains("alpha"));
    assert!(cell_string(&buffer, 1).contains(">>"));
    assert!(cell_string(&buffer, 1).contains("beta"));
    assert!(cell_string(&buffer, 2).contains("gamma"));
}

#[test]
fn list_scroll_window_accounts_for_multiline_item_heights() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::List(ListProps {
            block: None,
            items: vec![
                ListItem::new("alpha\nbeta"),
                ListItem::new("gamma"),
                ListItem::new("delta"),
            ],
            state: ListState::default().with_selected(Some(2)),
            highlight_symbol: Some(Line::from(">>")),
            highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::Always,
            repeat_highlight_symbol: true,
            direction: ListDirection::TopToBottom,
            scroll_padding: 0,
            on_select: None,
        }))
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 12, 2),
    );

    let mut buffer = FrameBuffer::new(12, 2);
    draw_tree(&tree, None, &mut buffer);

    assert!(cell_string(&buffer, 0).contains("gamma"));
    assert!(cell_string(&buffer, 1).contains("delta"));
}

#[test]
fn list_scroll_padding_accounts_for_multiline_item_heights_like_ratatui() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::List(ListProps {
            block: None,
            items: vec![
                ListItem::new("alpha"),
                ListItem::new("beta one\nbeta two"),
                ListItem::new("gamma"),
            ],
            state: ListState::default().with_selected(Some(2)),
            highlight_symbol: Some(Line::from(">>")),
            highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::Always,
            repeat_highlight_symbol: true,
            direction: ListDirection::TopToBottom,
            scroll_padding: 1,
            on_select: None,
        }))
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 18, 3),
    );

    let mut buffer = FrameBuffer::new(18, 3);
    draw_tree(&tree, None, &mut buffer);

    assert!(cell_string(&buffer, 0).contains("beta one"));
    assert!(cell_string(&buffer, 1).contains("beta two"));
    assert!(cell_string(&buffer, 2).contains(">>"));
    assert!(cell_string(&buffer, 2).contains("gamma"));
    assert!(!cell_string(&buffer, 0).contains("alpha"));
}

#[test]
fn paragraph_supports_vertical_scroll_without_reflowing_into_new_lines() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Paragraph(ParagraphProps {
            content: "line one\nline two\nline three".into(),
            block: None,
            alignment: Alignment::Left,
            wrap: None,
            scroll_x: 0,
            scroll_y: 1,
        }))
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 12, 2),
    );

    let mut buffer = FrameBuffer::new(12, 2);
    draw_tree(&tree, None, &mut buffer);

    assert!(cell_string(&buffer, 0).starts_with("line two"));
    assert!(cell_string(&buffer, 1).starts_with("line three"));
}

#[test]
fn paragraph_wraps_on_word_boundaries_like_ratatui() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Paragraph(ParagraphProps {
            content: Text::from("foo bar baz"),
            block: None,
            alignment: Alignment::Left,
            wrap: Some(Wrap { trim: true }),
            scroll_x: 0,
            scroll_y: 0,
        })),
        Rect::new(0, 0, 6, 3),
    );

    let mut buffer = FrameBuffer::new(6, 3);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(cell_string(&buffer, 0), "foo   ");
    assert_eq!(cell_string(&buffer, 1), "bar   ");
    assert_eq!(cell_string(&buffer, 2), "baz   ");
}

#[test]
fn paragraph_renders_styled_spans_with_alignment() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Paragraph(ParagraphProps {
            content: Text::from(Line::from(vec![
                Span::raw("hi"),
                Span::raw(" "),
                Span::styled("there", Style::default().fg(Color::Cyan)),
            ]))
            .centered(),
            block: None,
            alignment: Alignment::Left,
            wrap: None,
            scroll_x: 0,
            scroll_y: 0,
        }))
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 14, 1),
    );

    let mut buffer = FrameBuffer::new(14, 1);
    draw_tree(&tree, None, &mut buffer);

    assert!(cell_string(&buffer, 0).starts_with("   hi there"));
    assert_eq!(buffer.get(6, 0).style.fg, Color::Cyan);
}

#[test]
fn paragraph_can_render_inside_an_embedded_block() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Paragraph(ParagraphProps {
            content: "line one\nline two".into(),
            block: Some(ansiq_core::BlockFrame {
                props: bordered_block_props("Paragraph"),
                style: Style::default().fg(Color::Grey),
            }),
            alignment: Alignment::Left,
            wrap: Some(Wrap { trim: false }),
            scroll_x: 0,
            scroll_y: 0,
        })),
        Rect::new(0, 0, 18, 6),
    );

    let mut buffer = FrameBuffer::new(18, 6);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(buffer.get(0, 0).symbol, '┌');
    assert!(cell_string(&buffer, 0).contains("Paragraph"));
    assert!(cell_string(&buffer, 2).contains("line one"));
    assert!(cell_string(&buffer, 3).contains("line two"));
}

#[test]
fn table_selection_uses_style_without_injecting_prefix_characters() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Table(TableProps {
            block: None,
            header: Some(Row::new(["Name", "PID"])),
            footer: None,
            rows: vec![Row::new(["alpha", "1"]), Row::new(["beta", "2"])],
            widths: vec![Constraint::Fill(1), Constraint::Fill(1)],
            column_spacing: 1,
            flex: ansiq_core::Flex::Start,
            alignments: vec![TableAlignment::Left, TableAlignment::Right],
            state: TableState::default().with_selected(Some(1)),
            highlight_symbol: None,
            row_highlight_style: Style::default(),
            column_highlight_style: Style::default(),
            cell_highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            on_select: None,
        }))
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 16, 3),
    );

    let mut buffer = FrameBuffer::new(16, 3);
    draw_tree(&tree, None, &mut buffer);

    assert!(cell_string(&buffer, 2).starts_with("beta"));
    assert!(!cell_string(&buffer, 2).contains(">"));
    assert!(buffer.get(0, 2).style.reversed);
}

#[test]
fn draw_tree_in_regions_updates_only_the_invalidated_area() {
    let before: Node<()> = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 1,
        }))
        .with_children(vec![
            Element::new(ElementKind::Text(ansiq_core::TextProps {
                content: "idle".to_string(),
            })),
            Element::new(ElementKind::Text(ansiq_core::TextProps {
                content: "stable".to_string(),
            })),
        ]),
        Rect::new(0, 0, 12, 4),
    );
    let after: Node<()> = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 1,
        }))
        .with_children(vec![
            Element::new(ElementKind::Text(ansiq_core::TextProps {
                content: "ready".to_string(),
            })),
            Element::new(ElementKind::Text(ansiq_core::TextProps {
                content: "stable".to_string(),
            })),
        ]),
        Rect::new(0, 0, 12, 4),
    );

    let mut previous = FrameBuffer::new(12, 4);
    draw_tree(&before, None, &mut previous);
    let mut next = previous.clone();
    draw_tree_in_regions(&after, None, &mut next, &[after.children[0].rect]);

    assert!(cell_string(&next, 0).starts_with("ready"));
    assert_eq!(cell_string(&next, 2), cell_string(&previous, 2));
}

#[test]
fn tabs_and_gauge_render_selection_and_progress() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Box(BoxProps {
            direction: Direction::Column,
            gap: 1,
        }))
        .with_children(vec![
            Element::new(ElementKind::Tabs(TabsProps {
                block: None,
                titles: vec![
                    "Overview".into(),
                    Line::from(vec![
                        Span::raw("Fi"),
                        Span::styled("les", Style::default().fg(Color::Cyan)),
                    ]),
                    "Logs".into(),
                ],
                selected: Some(1),
                selection_explicit: false,
                highlight_style: Style::default().fg(Color::Yellow),
                divider: Span::raw("|"),
                padding_left: Line::from(" "),
                padding_right: Line::from(" "),
                on_select: None,
            })),
            Element::new(ElementKind::Gauge(GaugeProps {
                block: None,
                ratio: 0.42,
                label: Some(Span::raw("42%")),
                use_unicode: true,
                gauge_style: Style::default().fg(Color::Cyan),
            })),
        ]),
        Rect::new(0, 0, 24, 4),
    );

    let mut buffer = FrameBuffer::new(24, 4);
    draw_tree(&tree, None, &mut buffer);

    assert!(cell_string(&buffer, 0).contains("Overview"));
    assert!(cell_string(&buffer, 0).contains("Files"));
    assert!(cell_string(&buffer, 0).contains("|"));
    assert!(buffer.get(13, 0).style.reversed || buffer.get(14, 0).style.reversed);
    assert_eq!(buffer.get(13, 0).style.fg, Color::Yellow);
    assert_eq!(buffer.get(15, 0).style.fg, Color::Cyan);
    let gauge_line = cell_string(&buffer, 2);
    assert!(gauge_line.contains("42%"));
    assert!(gauge_line.starts_with("█"));
    assert_eq!(buffer.get(0, 2).style.fg, Color::Cyan);
}

#[test]
fn tabs_can_render_inside_an_embedded_block() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Tabs(TabsProps {
            block: Some(ansiq_core::BlockFrame {
                props: bordered_block_props("Tabs"),
                style: Style::default().fg(Color::Grey),
            }),
            titles: vec!["Overview".into(), "Files".into()],
            selected: Some(1),
            selection_explicit: false,
            highlight_style: Style::default().fg(Color::Yellow),
            divider: Span::raw("|"),
            padding_left: Line::from(" "),
            padding_right: Line::from(" "),
            on_select: None,
        })),
        Rect::new(0, 0, 24, 5),
    );

    let mut buffer = FrameBuffer::new(24, 5);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(buffer.get(0, 0).symbol, '┌');
    assert!(cell_string(&buffer, 0).contains("Tabs"));
    assert!(cell_string(&buffer, 2).contains("Overview"));
    assert!(cell_string(&buffer, 2).contains("Files"));
    assert!(
        cell_string(&buffer, 2).trim_start().starts_with("Overview")
            || cell_string(&buffer, 2).contains("Files")
    );
}

#[test]
fn gauge_can_use_unicode_fractional_blocks_when_no_label_overlays_them() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Gauge(GaugeProps {
            block: None,
            ratio: 0.37,
            label: Some(Span::raw("")),
            use_unicode: true,
            gauge_style: Style::default().fg(Color::Cyan),
        })),
        Rect::new(0, 0, 10, 1),
    );

    let mut buffer = FrameBuffer::new(10, 1);
    draw_tree(&tree, None, &mut buffer);

    let line = cell_string(&buffer, 0);
    assert!(
        line.contains("▏")
            || line.contains("▎")
            || line.contains("▍")
            || line.contains("▌")
            || line.contains("▋")
            || line.contains("▊")
            || line.contains("▉")
    );
}

#[test]
fn table_renders_headers_rows_and_selected_state() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Table(TableProps {
            block: None,
            header: Some(Row::new(["Name", "Status"])),
            footer: None,
            rows: vec![
                Row::new(["ansiq", "ready"]),
                Row::new(["codexdemo", "streaming"]),
            ],
            widths: vec![Constraint::Fill(1), Constraint::Fill(1)],
            column_spacing: 1,
            flex: ansiq_core::Flex::Start,
            alignments: vec![TableAlignment::Left, TableAlignment::Left],
            state: TableState::default().with_selected(Some(1)),
            highlight_symbol: None,
            row_highlight_style: Style::default(),
            column_highlight_style: Style::default(),
            cell_highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            on_select: None,
        }))
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 24, 4),
    );

    let mut buffer = FrameBuffer::new(24, 4);
    draw_tree(&tree, None, &mut buffer);

    assert!(cell_string(&buffer, 0).contains("Name"));
    assert!(cell_string(&buffer, 0).contains("Status"));
    assert!(cell_string(&buffer, 1).contains("ansiq"));
    assert!(cell_string(&buffer, 2).contains("codexdemo"));
    assert!(!cell_string(&buffer, 2).contains(">"));
    assert!(buffer.get(0, 2).style.bold);
    assert!(buffer.get(0, 2).style.reversed);
}

#[test]
fn table_auto_scrolls_selected_row_and_reserves_highlight_gutter() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Table(TableProps {
            block: None,
            header: Some(Row::new(["Name", "Status"])),
            footer: None,
            rows: vec![
                Row::new(["ansiq", "ready"]),
                Row::new(["codexdemo", "streaming"]),
                Row::new(["agent", "idle"]),
                Row::new(["worker", "busy"]),
            ],
            widths: vec![Constraint::Fill(1), Constraint::Fill(1)],
            column_spacing: 1,
            flex: ansiq_core::Flex::Start,
            alignments: vec![TableAlignment::Left, TableAlignment::Left],
            state: TableState::default().with_selected(Some(3)),
            highlight_symbol: Some(">>".into()),
            row_highlight_style: Style::default(),
            column_highlight_style: Style::default(),
            cell_highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            on_select: None,
        }))
        .with_style(Style::default().fg(Color::Grey))
        .with_layout(Layout {
            width: Length::Fill,
            height: Length::Fixed(3),
        }),
        Rect::new(0, 0, 24, 3),
    );

    let mut buffer = FrameBuffer::new(24, 3);
    draw_tree(&tree, None, &mut buffer);

    assert!(cell_string(&buffer, 1).contains("agent"));
    assert!(cell_string(&buffer, 2).contains(">>"));
    assert!(cell_string(&buffer, 2).contains("worker"));
    assert!(buffer.get(0, 2).style.bold);
    assert!(buffer.get(0, 2).style.reversed);
}

#[test]
fn table_distributes_column_widths_clips_cells_and_right_aligns_numeric_columns() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Table(TableProps {
            block: None,
            header: Some(Row::new(["Task", "Ms"])),
            footer: None,
            rows: vec![Row::new(["builder-long", "42"])],
            widths: vec![Constraint::Fill(1), Constraint::Fill(1)],
            column_spacing: 1,
            flex: ansiq_core::Flex::Start,
            alignments: vec![TableAlignment::Left, TableAlignment::Right],
            state: TableState::default(),
            highlight_symbol: None,
            row_highlight_style: Style::default(),
            column_highlight_style: Style::default(),
            cell_highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            on_select: None,
        }))
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 15, 3),
    );

    let mut buffer = FrameBuffer::new(15, 3);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(cell_string(&buffer, 0), "Task         Ms");
    assert_eq!(cell_string(&buffer, 1), "builder      42");
}

#[test]
fn table_without_explicit_widths_uses_equal_columns_like_ratatui() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Table(TableProps {
            block: None,
            header: None,
            footer: None,
            rows: vec![Row::new(["Task", "Status"]), Row::new(["build", "ready"])],
            widths: vec![],
            column_spacing: 1,
            flex: ansiq_core::Flex::Start,
            alignments: vec![TableAlignment::Left, TableAlignment::Left],
            state: TableState::default(),
            highlight_symbol: None,
            row_highlight_style: Style::default(),
            column_highlight_style: Style::default(),
            cell_highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            on_select: None,
        })),
        Rect::new(0, 0, 16, 2),
    );

    let mut buffer = FrameBuffer::new(16, 2);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(cell_string(&buffer, 0), "Task     Status ");
    assert_eq!(cell_string(&buffer, 1), "build    ready  ");
}

#[test]
fn table_flex_start_keeps_extra_space_after_the_columns_like_ratatui() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Table(TableProps {
            block: None,
            header: None,
            footer: None,
            rows: vec![Row::new(["Task", "Status"])],
            widths: vec![Constraint::Length(4), Constraint::Length(6)],
            column_spacing: 1,
            alignments: vec![TableAlignment::Left, TableAlignment::Left],
            state: TableState::default(),
            highlight_symbol: None,
            row_highlight_style: Style::default(),
            column_highlight_style: Style::default(),
            cell_highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            on_select: None,
            flex: ansiq_core::Flex::Start,
        })),
        Rect::new(0, 0, 20, 1),
    );

    let mut buffer = FrameBuffer::new(20, 1);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(cell_string(&buffer, 0), "Task Status         ");
}

#[test]
fn table_flex_center_centers_columns_like_ratatui() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Table(TableProps {
            block: None,
            header: None,
            footer: None,
            rows: vec![Row::new(["Task", "Status"])],
            widths: vec![Constraint::Length(4), Constraint::Length(6)],
            column_spacing: 1,
            flex: ansiq_core::Flex::Center,
            alignments: vec![TableAlignment::Left, TableAlignment::Left],
            state: TableState::default(),
            highlight_symbol: None,
            row_highlight_style: Style::default(),
            column_highlight_style: Style::default(),
            cell_highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            on_select: None,
        })),
        Rect::new(0, 0, 20, 1),
    );

    let mut buffer = FrameBuffer::new(20, 1);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(cell_string(&buffer, 0), "    Task Status     ");
}

#[test]
fn table_flex_end_pushes_columns_to_the_right_like_ratatui() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Table(TableProps {
            block: None,
            header: None,
            footer: None,
            rows: vec![Row::new(["Task", "Status"])],
            widths: vec![Constraint::Length(4), Constraint::Length(6)],
            column_spacing: 1,
            flex: ansiq_core::Flex::End,
            alignments: vec![TableAlignment::Left, TableAlignment::Left],
            state: TableState::default(),
            highlight_symbol: None,
            row_highlight_style: Style::default(),
            column_highlight_style: Style::default(),
            cell_highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            on_select: None,
        })),
        Rect::new(0, 0, 20, 1),
    );

    let mut buffer = FrameBuffer::new(20, 1);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(cell_string(&buffer, 0), "         Task Status");
}

#[test]
fn table_flex_space_between_distributes_extra_width_between_columns_like_ratatui() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Table(TableProps {
            block: None,
            header: None,
            footer: None,
            rows: vec![Row::new(["Task", "Status"])],
            widths: vec![Constraint::Length(4), Constraint::Length(6)],
            column_spacing: 1,
            flex: ansiq_core::Flex::SpaceBetween,
            alignments: vec![TableAlignment::Left, TableAlignment::Left],
            state: TableState::default(),
            highlight_symbol: None,
            row_highlight_style: Style::default(),
            column_highlight_style: Style::default(),
            cell_highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            on_select: None,
        })),
        Rect::new(0, 0, 20, 1),
    );

    let mut buffer = FrameBuffer::new(20, 1);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(cell_string(&buffer, 0), "Task          Status");
}

#[test]
fn table_flex_space_around_distributes_extra_width_around_columns_like_ratatui() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Table(TableProps {
            block: None,
            header: None,
            footer: None,
            rows: vec![Row::new(["Task", "Status"])],
            widths: vec![Constraint::Length(4), Constraint::Length(6)],
            column_spacing: 1,
            flex: ansiq_core::Flex::SpaceAround,
            alignments: vec![TableAlignment::Left, TableAlignment::Left],
            state: TableState::default(),
            highlight_symbol: None,
            row_highlight_style: Style::default(),
            column_highlight_style: Style::default(),
            cell_highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            on_select: None,
        })),
        Rect::new(0, 0, 20, 1),
    );

    let mut buffer = FrameBuffer::new(20, 1);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(cell_string(&buffer, 0), "  Task      Status  ");
}

#[test]
fn table_flex_space_evenly_distributes_extra_width_evenly_like_ratatui() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Table(TableProps {
            block: None,
            header: None,
            footer: None,
            rows: vec![Row::new(["Task", "Status"])],
            widths: vec![Constraint::Length(4), Constraint::Length(6)],
            column_spacing: 1,
            flex: ansiq_core::Flex::SpaceEvenly,
            alignments: vec![TableAlignment::Left, TableAlignment::Left],
            state: TableState::default(),
            highlight_symbol: None,
            row_highlight_style: Style::default(),
            column_highlight_style: Style::default(),
            cell_highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            on_select: None,
        })),
        Rect::new(0, 0, 20, 1),
    );

    let mut buffer = FrameBuffer::new(20, 1);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(cell_string(&buffer, 0), "   Task    Status   ");
}

#[test]
fn table_renders_multiline_cells_without_manual_row_height_like_ratatui() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Table(TableProps {
            block: None,
            header: None,
            footer: None,
            rows: vec![Row::new(["Cell1\nCell2", "Value"])],
            widths: vec![Constraint::Length(10), Constraint::Length(8)],
            column_spacing: 1,
            flex: ansiq_core::Flex::Start,
            alignments: vec![TableAlignment::Left, TableAlignment::Left],
            state: TableState::default(),
            highlight_symbol: None,
            row_highlight_style: Style::default(),
            column_highlight_style: Style::default(),
            cell_highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            on_select: None,
        })),
        Rect::new(0, 0, 19, 2),
    );

    let mut buffer = FrameBuffer::new(19, 2);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(cell_string(&buffer, 0), "Cell1      Value   ");
    assert_eq!(cell_string(&buffer, 1), "Cell2              ");
}

#[test]
fn table_prioritizes_highlight_spacing_over_column_constraints_like_ratatui() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Table(TableProps {
            block: None,
            header: None,
            footer: None,
            rows: vec![Row::new(["ABCDE", "12345"])],
            widths: vec![Constraint::Length(5), Constraint::Length(5)],
            column_spacing: 1,
            flex: ansiq_core::Flex::Start,
            alignments: vec![TableAlignment::Left, TableAlignment::Left],
            state: TableState::default(),
            highlight_symbol: Some(">>>".into()),
            row_highlight_style: Style::default(),
            column_highlight_style: Style::default(),
            cell_highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::Always,
            on_select: None,
        })),
        Rect::new(0, 0, 10, 1),
    );

    let mut buffer = FrameBuffer::new(10, 1);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(cell_string(&buffer, 0), "   ABCD123");
}

#[test]
fn table_can_render_inside_an_embedded_block_with_footer() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Table(TableProps {
            block: Some(ansiq_core::BlockFrame {
                props: bordered_block_props("Processes"),
                style: Style::default().fg(Color::Grey),
            }),
            header: Some(Row::new(["Name", "PID"])),
            footer: Some(Row::new(["Summary", "2 rows"])),
            rows: vec![Row::new(["alpha", "1"]), Row::new(["beta", "2"])],
            widths: vec![Constraint::Length(10), Constraint::Length(8)],
            column_spacing: 2,
            flex: ansiq_core::Flex::Start,
            alignments: vec![TableAlignment::Left, TableAlignment::Right],
            state: TableState::default().with_selected(Some(1)),
            highlight_symbol: None,
            row_highlight_style: Style::default(),
            column_highlight_style: Style::default(),
            cell_highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            on_select: None,
        })),
        Rect::new(0, 0, 24, 8),
    );

    let mut buffer = FrameBuffer::new(24, 8);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(buffer.get(0, 0).symbol, '┌');
    assert!(cell_string(&buffer, 0).contains("Processes"));
    assert!(cell_string(&buffer, 2).contains("Name"));
    assert!(cell_string(&buffer, 3).contains("alpha"));
    assert!(cell_string(&buffer, 4).contains("beta"));
    assert!(cell_string(&buffer, 5).contains("Summary"));
}

#[test]
fn table_min_constraints_expand_under_flex_start_like_ratatui() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Table(TableProps {
            block: None,
            header: None,
            footer: None,
            rows: vec![Row::new(["A", "B", "C"])],
            widths: vec![Constraint::Min(10), Constraint::Min(10), Constraint::Min(1)],
            column_spacing: 1,
            flex: ansiq_core::Flex::Start,
            alignments: vec![
                TableAlignment::Left,
                TableAlignment::Left,
                TableAlignment::Left,
            ],
            state: TableState::default(),
            highlight_symbol: None,
            row_highlight_style: Style::default(),
            column_highlight_style: Style::default(),
            cell_highlight_style: Style::default(),
            highlight_spacing: HighlightSpacing::WhenSelected,
            on_select: None,
        })),
        Rect::new(0, 0, 62, 1),
    );

    let mut buffer = FrameBuffer::new(62, 1);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(buffer.get(0, 0).symbol, 'A');
    assert_eq!(buffer.get(21, 0).symbol, 'B');
    assert_eq!(buffer.get(42, 0).symbol, 'C');
}

#[test]
fn scrollbar_renders_a_vertical_thumb_inside_the_track() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Scrollbar(ScrollbarProps {
            state: ScrollbarState::new(12)
                .position(6)
                .viewport_content_length(3),
            orientation: ansiq_core::ScrollbarOrientation::VerticalRight,
            thumb_symbol: "█".to_string(),
            thumb_style: Style::default().fg(Color::White),
            track_symbol: Some("░".to_string()),
            track_style: Style::default().fg(Color::DarkGrey),
            begin_symbol: Some("↑".to_string()),
            begin_style: Style::default().fg(Color::Yellow),
            end_symbol: Some("↓".to_string()),
            end_style: Style::default().fg(Color::Yellow),
            on_scroll: None,
        }))
        .with_layout(Layout {
            width: Length::Fixed(1),
            height: Length::Fill,
        })
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 1, 6),
    );

    let mut buffer = FrameBuffer::new(1, 6);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(buffer.get(0, 0).symbol, '↑');
    assert_eq!(buffer.get(0, 1).symbol, '░');
    assert_eq!(buffer.get(0, 2).symbol, '░');
    assert_eq!(buffer.get(0, 3).symbol, '█');
    assert_eq!(buffer.get(0, 4).symbol, '░');
    assert_eq!(buffer.get(0, 5).symbol, '↓');
    assert_eq!(buffer.get(0, 0).style.fg, Color::Yellow);
    assert_eq!(buffer.get(0, 3).style.fg, Color::White);
    assert_eq!(buffer.get(0, 1).style.fg, Color::DarkGrey);
}

#[test]
fn scrollbar_with_zero_content_length_renders_blank_like_ratatui() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Scrollbar(ScrollbarProps {
            state: ScrollbarState::default(),
            orientation: ansiq_core::ScrollbarOrientation::VerticalRight,
            thumb_symbol: "█".to_string(),
            thumb_style: Style::default().fg(Color::White),
            track_symbol: Some("░".to_string()),
            track_style: Style::default().fg(Color::DarkGrey),
            begin_symbol: Some("↑".to_string()),
            begin_style: Style::default().fg(Color::Yellow),
            end_symbol: Some("↓".to_string()),
            end_style: Style::default().fg(Color::Yellow),
            on_scroll: None,
        }))
        .with_layout(Layout {
            width: Length::Fixed(1),
            height: Length::Fill,
        })
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 1, 4),
    );

    let mut buffer = FrameBuffer::new(1, 4);
    draw_tree(&tree, None, &mut buffer);

    for y in 0..4 {
        assert_eq!(buffer.get(0, y).symbol, ' ');
    }
}

#[test]
fn scrollbar_renders_a_horizontal_thumb_inside_the_track() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Scrollbar(ScrollbarProps {
            state: ScrollbarState::new(12)
                .position(6)
                .viewport_content_length(3),
            orientation: ansiq_core::ScrollbarOrientation::HorizontalBottom,
            thumb_symbol: "█".to_string(),
            thumb_style: Style::default().fg(Color::White),
            track_symbol: Some("░".to_string()),
            track_style: Style::default().fg(Color::DarkGrey),
            begin_symbol: Some("←".to_string()),
            begin_style: Style::default().fg(Color::Yellow),
            end_symbol: Some("→".to_string()),
            end_style: Style::default().fg(Color::Yellow),
            on_scroll: None,
        }))
        .with_layout(Layout {
            width: Length::Fill,
            height: Length::Fixed(1),
        })
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 6, 1),
    );

    let mut buffer = FrameBuffer::new(6, 1);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(cell_string(&buffer, 0), "←░░█░→");
    assert_eq!(buffer.get(0, 0).style.fg, Color::Yellow);
    assert_eq!(buffer.get(3, 0).style.fg, Color::White);
    assert_eq!(buffer.get(1, 0).style.fg, Color::DarkGrey);
}

#[test]
fn clear_erases_existing_content_and_line_gauge_renders_a_thin_track() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Pane(PaneProps { title: None })).with_children(vec![
            Element::new(ElementKind::Paragraph(ParagraphProps {
                content: "busy".into(),
                block: None,
                alignment: Alignment::Left,
                wrap: Some(Wrap { trim: false }),
                scroll_x: 0,
                scroll_y: 0,
            })),
            Element::new(ElementKind::Clear(ClearProps)),
        ]),
        Rect::new(0, 0, 10, 4),
    );

    let mut buffer = FrameBuffer::new(10, 4);
    draw_tree(&tree, None, &mut buffer);
    assert_eq!(buffer.get(0, 1).symbol, '│');
    assert_eq!(buffer.get(9, 1).symbol, '│');
    let inner: String = cell_string(&buffer, 1).chars().skip(1).take(8).collect();
    assert_eq!(inner.trim(), "");

    let line_gauge: Node<()> = layout_tree(
        Element::new(ElementKind::LineGauge(LineGaugeProps {
            block: None,
            ratio: 0.58,
            label: Some(Line::from("58%")),
            filled_symbol: "=".to_string(),
            unfilled_symbol: ".".to_string(),
            filled_style: Style::default().fg(Color::Green),
            unfilled_style: Style::default().fg(Color::DarkGrey),
        }))
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 16, 1),
    );

    let mut gauge_buffer = FrameBuffer::new(16, 1);
    draw_tree(&line_gauge, None, &mut gauge_buffer);
    assert!(cell_string(&gauge_buffer, 0).contains("58%"));
    assert!(cell_string(&gauge_buffer, 0).contains("="));
    assert!(cell_string(&gauge_buffer, 0).contains("."));
    assert_eq!(gauge_buffer.get(4, 0).style.fg, Color::Green);
}

#[test]
fn sparkline_renders_single_line_bars_from_values() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Sparkline(SparklineProps {
            values: vec![Some(1), Some(2), Some(4), Some(6), Some(8)],
            max: Some(8),
            direction: ansiq_core::SparklineDirection::LeftToRight,
            absent_value_symbol: '·',
            absent_value_style: Style::default().fg(Color::DarkGrey),
        }))
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 5, 1),
    );

    let mut buffer = FrameBuffer::new(5, 1);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(cell_string(&buffer, 0), "▁▂▄▆█");
}

#[test]
fn sparkline_supports_missing_values_and_reverse_direction() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Sparkline(SparklineProps {
            values: vec![Some(1), None, Some(8)],
            max: Some(8),
            direction: ansiq_core::SparklineDirection::RightToLeft,
            absent_value_symbol: '·',
            absent_value_style: Style::default().fg(Color::DarkGrey),
        }))
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 3, 1),
    );

    let mut buffer = FrameBuffer::new(3, 1);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(cell_string(&buffer, 0), "█·▁");
    assert_eq!(buffer.get(1, 0).style.fg, Color::DarkGrey);
}

#[test]
fn barchart_renders_columns_with_labels() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::BarChart(BarChartProps {
            bars: vec![
                Bar {
                    label: "cpu".to_string(),
                    value: 80,
                },
                Bar {
                    label: "mem".to_string(),
                    value: 40,
                },
            ],
            max: Some(100),
            bar_width: 3,
        }))
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 14, 6),
    );

    let mut buffer = FrameBuffer::new(14, 6);
    draw_tree(&tree, None, &mut buffer);

    assert!(cell_string(&buffer, 5).contains("cpu"));
    assert!(cell_string(&buffer, 4).contains("█"));
}

#[test]
fn chart_renders_axes_and_points() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Chart(ChartProps {
            datasets: vec![ChartDataset {
                label: Some("cpu".to_string()),
                points: vec![(0, 1), (1, 3), (2, 2), (3, 4)],
            }],
            min_y: Some(0),
            max_y: Some(4),
        }))
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 16, 8),
    );

    let mut buffer = FrameBuffer::new(16, 8);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(buffer.get(0, 0).symbol, '│');
    assert!(cell_string(&buffer, 7).contains("─"));
    assert!(cell_string(&buffer, 1).contains("•") || cell_string(&buffer, 2).contains("•"));
}

#[test]
fn canvas_renders_cells_at_declared_positions() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Canvas(CanvasProps {
            width: 8,
            height: 4,
            cells: vec![
                CanvasCell {
                    x: 1,
                    y: 1,
                    symbol: '•',
                    style: Style::default().fg(Color::Cyan),
                },
                CanvasCell {
                    x: 4,
                    y: 2,
                    symbol: 'x',
                    style: Style::default().fg(Color::Grey),
                },
            ],
        })),
        Rect::new(0, 0, 12, 8),
    );

    let mut buffer = FrameBuffer::new(12, 8);
    draw_tree(&tree, None, &mut buffer);

    assert_eq!(buffer.get(1, 1).symbol, '•');
    assert_eq!(buffer.get(4, 2).symbol, 'x');
}

#[test]
fn monthly_renders_title_weekdays_and_selected_day() {
    let tree: Node<()> = layout_tree(
        Element::new(ElementKind::Monthly(MonthlyProps {
            year: 2026,
            month: 4,
            selected_day: Some(4),
        }))
        .with_style(Style::default().fg(Color::Grey)),
        Rect::new(0, 0, 24, 8),
    );

    let mut buffer = FrameBuffer::new(24, 8);
    draw_tree(&tree, None, &mut buffer);

    assert!(cell_string(&buffer, 0).contains("2026-04"));
    assert!(cell_string(&buffer, 1).contains("Mo Tu We Th Fr Sa Su"));
    assert!(cell_string(&buffer, 2).contains(" 4"));
}

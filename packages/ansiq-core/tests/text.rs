use ansiq_core::{
    Alignment, Color, Line, Span, Style, Text, clip_to_width, display_width, display_width_prefix,
    styled_line_from_line, styled_lines_from_text, wrap_plain_lines, wrap_styled_lines,
};

#[test]
fn display_width_counts_wide_characters() {
    assert_eq!(display_width("大河"), 4);
    assert_eq!(display_width_prefix("大河向东流", 2), 4);
}

#[test]
fn clip_to_width_respects_display_width_boundaries() {
    assert_eq!(clip_to_width("abcdef", 4), "abcd");
    assert_eq!(clip_to_width("你好ab", 4), "你好");
}

#[test]
fn wrap_plain_lines_splits_long_lines_without_dropping_empty_lines() {
    assert_eq!(
        wrap_plain_lines("abcd\nefghij", 4, false),
        vec!["abcd".to_string(), "efgh".to_string(), "ij".to_string()]
    );
    assert_eq!(
        wrap_plain_lines("a\n\nb", 2, false),
        vec!["a".to_string(), String::new(), "b".to_string()]
    );
}

#[test]
fn styled_lines_from_text_patches_span_styles_into_the_base_style() {
    let text = Text::from(vec![Line::from(vec![
        Span::styled("A", Style::default().fg(Color::Yellow)),
        Span::raw("b"),
    ])])
    .centered();

    let lines = styled_lines_from_text(&text, Style::default().bg(Color::Blue), Alignment::Left);

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].alignment, Alignment::Center);
    assert_eq!(lines[0].chunks[0].style.fg, Color::Yellow);
    assert_eq!(lines[0].chunks[0].style.bg, Color::Blue);
    assert_eq!(lines[0].chunks[1].style.fg, Color::Reset);
    assert_eq!(lines[0].chunks[1].style.bg, Color::Blue);
}

#[test]
fn wrap_styled_lines_wraps_by_tokens_and_preserves_styles() {
    let line = styled_line_from_line(
        &Line::from(vec![
            Span::styled("hello", Style::default().fg(Color::Green)),
            Span::raw(" "),
            Span::styled("world", Style::default().fg(Color::Cyan)),
        ]),
        Style::default(),
    );

    let wrapped = wrap_styled_lines(&[line], 6, true);

    assert_eq!(wrapped.len(), 2);
    assert_eq!(wrapped[0].chunks[0].text, "hello");
    assert_eq!(wrapped[0].chunks[0].style.fg, Color::Green);
    assert_eq!(wrapped[1].chunks[0].text, "world");
    assert_eq!(wrapped[1].chunks[0].style.fg, Color::Cyan);
}

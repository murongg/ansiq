use ansiq_core::{Color, Style};

#[test]
fn style_builder_chains_attributes() {
    let style = Style::default()
        .fg(Color::Cyan)
        .bg(Color::Black)
        .bold(true)
        .reversed(true);

    assert_eq!(style.fg, Color::Cyan);
    assert_eq!(style.bg, Color::Black);
    assert!(style.bold);
    assert!(style.reversed);
}

#[test]
fn style_supports_indexed_and_truecolor_variants() {
    let indexed = Style::default().fg(Color::Indexed(202));
    let rgb = Style::default().bg(Color::Rgb(12, 34, 56));

    assert_eq!(indexed.fg, Color::Indexed(202));
    assert_eq!(rgb.bg, Color::Rgb(12, 34, 56));
}

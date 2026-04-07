use ansiq_core::{Cell, Color, Line, Row, Span, Style, TableState};

#[test]
fn cell_can_wrap_text_style_and_column_span() {
    let cell = Cell::new(Line::from(vec![
        Span::raw("12"),
        Span::styled("3", Style::default().fg(Color::Yellow)),
    ]))
    .style(Style::default().fg(Color::Cyan))
    .column_span(2);

    assert_eq!(cell.width(), 3);
    assert_eq!(cell.height(), 1);
    assert_eq!(cell.column_span_value(), 2);
    assert_eq!(cell.style_value().fg, Color::Cyan);
}

#[test]
fn row_tracks_cells_height_and_margins() {
    let row = Row::new([Cell::new("cpu"), Cell::new("42")])
        .height(2)
        .top_margin(1)
        .bottom_margin(1)
        .style(Style::default().bold(true));

    assert_eq!(
        row,
        Row::new([Cell::new("cpu"), Cell::new("42")])
            .height(2)
            .top_margin(1)
            .bottom_margin(1)
            .style(Style::default().bold(true))
    );
    assert_eq!(row.height_with_margin(), 4);
}

#[test]
fn row_defaults_height_to_tallest_cell_like_ratatui() {
    let row = Row::new([Cell::new("cpu\nuser"), Cell::new("42")]);

    assert_eq!(row.height_value(), 2);
    assert_eq!(row.height_with_margin(), 2);
}

#[test]
fn row_collects_from_iterator_and_cell_style_accepts_into_style() {
    let row: Row = ["alpha", "beta", "gamma"].into_iter().collect();
    assert_eq!(
        row.cells_ref(),
        &[Cell::new("alpha"), Cell::new("beta"), Cell::new("gamma")]
    );

    let cell = Cell::new("accent").style(Color::Yellow);
    assert_eq!(cell.style_value().fg, Color::Yellow);
}

#[test]
fn table_state_supports_selected_cell_and_directional_navigation() {
    let mut state = TableState::new().with_selected_cell(Some((2, 3)));
    assert_eq!(state.selected(), Some(2));
    assert_eq!(state.selected_column(), Some(3));
    assert_eq!(state.selected_cell(), Some((2, 3)));

    state.select_next();
    state.select_next_column();
    assert_eq!(state.selected_cell(), Some((3, 4)));

    state.scroll_up_by(2);
    state.scroll_left_by(3);
    assert_eq!(state.selected_cell(), Some((1, 1)));

    state.select_cell(None);
    assert_eq!(state.selected(), None);
    assert_eq!(state.selected_column(), None);
    assert_eq!(state.selected_cell(), None);
    assert_eq!(state.offset(), 0);
}

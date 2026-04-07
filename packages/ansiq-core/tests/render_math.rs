use ansiq_core::{
    Constraint, Flex, TitleGroupPositions, table_column_layout, table_span_width,
    title_group_positions,
};

#[test]
fn title_group_positions_preserve_left_and_right_and_drop_center_when_space_is_tight() {
    let positions = title_group_positions(10, 4, 4, 4);

    assert_eq!(
        positions,
        TitleGroupPositions {
            left_x: Some(0),
            center_x: None,
            right_x: Some(6),
        }
    );
}

#[test]
fn table_column_layout_matches_ratatui_space_evenly_distribution() {
    let (widths, positions) = table_column_layout(
        20,
        3,
        &[
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ],
        1,
        Flex::SpaceEvenly,
    );

    assert_eq!(widths, vec![3, 3, 3]);
    assert_eq!(positions, vec![3, 9, 15]);
}

#[test]
fn table_span_width_uses_actual_column_positions_instead_of_assuming_fixed_spacing() {
    let width = table_span_width(&[4, 5, 3], &[0, 6, 13], 20, 0, 2);

    assert_eq!(width, 11);
}

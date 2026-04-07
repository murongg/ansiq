use ansiq_core::Rect;

#[test]
fn shrink_moves_origin_even_when_rect_collapses() {
    let rect = Rect::new(2, 3, 2, 2).shrink(1);

    assert_eq!(rect, Rect::new(3, 4, 0, 0));
}

#[test]
fn intersection_returns_the_overlap_between_two_rects() {
    let overlap = Rect::new(0, 0, 4, 4).intersection(Rect::new(2, 1, 4, 4));

    assert_eq!(overlap, Some(Rect::new(2, 1, 2, 3)));
}

#[test]
fn contains_and_union_cover_nested_and_overlapping_rects() {
    let outer = Rect::new(0, 0, 6, 6);
    let inner = Rect::new(2, 2, 2, 2);
    let merged = Rect::new(0, 0, 4, 4).union(Rect::new(2, 2, 4, 3));

    assert!(outer.contains(inner));
    assert_eq!(merged, Rect::new(0, 0, 6, 5));
}

#[test]
fn can_merge_rect_allows_axis_aligned_adjacent_regions_but_not_corner_contacts() {
    let top = Rect::new(0, 0, 12, 2);
    let below = Rect::new(0, 2, 12, 1);
    let left = Rect::new(0, 0, 4, 3);
    let right = Rect::new(4, 0, 2, 3);
    let corner = Rect::new(12, 2, 2, 2);

    assert!(top.can_merge_rect(below));
    assert!(left.can_merge_rect(right));
    assert!(!top.can_merge_rect(corner));
}

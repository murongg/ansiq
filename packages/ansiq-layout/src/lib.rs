use ansiq_core::{ChildLayoutKind, Direction, Element, Length, Node, Rect};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RelayoutStats {
    pub remeasured_nodes: usize,
    pub repositioned_nodes: usize,
    pub invalidated_regions: Vec<Rect>,
}

pub fn layout_tree<Message>(element: Element<Message>, bounds: Rect) -> Node<Message> {
    let mut next_id = 0;
    layout_tree_with_ids(element, bounds, &mut next_id)
}

pub fn layout_tree_with_ids<Message>(
    element: Element<Message>,
    bounds: Rect,
    next_id: &mut usize,
) -> Node<Message> {
    layout_node(element, bounds, next_id)
}

pub fn measure_height<Message>(element: &Element<Message>, width: u16) -> u16 {
    measured_height(element, width).max(1)
}

pub fn relayout_tree<Message>(node: &mut Node<Message>, bounds: Rect) {
    relayout_node(node, bounds);
}

pub fn relayout_tree_along_paths<Message>(
    node: &mut Node<Message>,
    bounds: Rect,
    dirty_paths: &[Vec<usize>],
) -> RelayoutStats {
    let mut stats = RelayoutStats::default();
    let normalized_dirty_paths = normalize_dirty_paths(dirty_paths);
    relayout_node_along_paths(
        node,
        bounds,
        &mut Vec::new(),
        &normalized_dirty_paths,
        &mut stats,
    );
    stats
}

pub fn measure_node_height<Message>(node: &Node<Message>, width: u16) -> u16 {
    measured_node_height_cached(node, width).max(1)
}

fn layout_node<Message>(
    element: Element<Message>,
    bounds: Rect,
    next_id: &mut usize,
) -> Node<Message> {
    let id = *next_id;
    *next_id += 1;

    let child_rects = child_rects(&element, bounds);
    let Element {
        kind,
        layout,
        style,
        focusable,
        continuity_key,
        children,
    } = element;
    let children = children
        .into_iter()
        .zip(child_rects)
        .map(|(child, rect)| layout_node(child, rect, next_id))
        .collect();

    let mut node = Node {
        id,
        rect: bounds,
        measured_height: 0,
        element: Element {
            kind,
            layout,
            style,
            focusable,
            continuity_key,
            children: Vec::new(),
        },
        children,
    };
    node.measured_height = remeasure_node_height(&node, bounds.width).max(1);
    node
}

fn relayout_node<Message>(node: &mut Node<Message>, bounds: Rect) {
    let rect_changed = node.rect != bounds;
    node.rect = bounds;
    node.measured_height = remeasure_node_height(node, bounds.width).max(1);

    let child_rects = child_rects_for_node(node, bounds);
    for (child, rect) in node.children.iter_mut().zip(child_rects) {
        if rect_changed || child.rect != rect {
            relayout_node(child, rect);
        }
    }
}

fn relayout_node_along_paths<Message>(
    node: &mut Node<Message>,
    bounds: Rect,
    path: &mut Vec<usize>,
    dirty_paths: &[Vec<usize>],
    stats: &mut RelayoutStats,
) {
    let old_rect = node.rect;
    let rect_changed = old_rect != bounds;
    let width_changed = old_rect.width != bounds.width;
    let affects_node = path_affects_node(dirty_paths, path);
    let invalidates_self = node.element.invalidates_self_on_layout_change();

    if rect_changed {
        node.rect = bounds;
        stats.repositioned_nodes += 1;
    }

    if path_is_dirty_target(dirty_paths, path) {
        push_invalidated_region(&mut stats.invalidated_regions, old_rect);
        push_invalidated_region(&mut stats.invalidated_regions, bounds);
    } else if rect_changed && invalidates_self {
        push_invalidated_region(&mut stats.invalidated_regions, old_rect);
        push_invalidated_region(&mut stats.invalidated_regions, bounds);
    } else if width_changed && invalidates_self {
        push_invalidated_region(&mut stats.invalidated_regions, bounds);
    }

    // Dirty subtree replacement already rebuilds the changed branch with valid
    // child caches. Here we only need to remeasure the changed node and its
    // ancestors, while rect-only shifts can reuse cached heights.
    if affects_node || width_changed {
        node.measured_height = remeasure_node_height(node, bounds.width).max(1);
        stats.remeasured_nodes += 1;
    }

    if !(rect_changed || affects_node || width_changed) {
        return;
    }

    let child_rects = child_rects_for_node(node, bounds);
    for (index, (child, rect)) in node.children.iter_mut().zip(child_rects).enumerate() {
        path.push(index);
        if child.rect != rect || path_affects_node(dirty_paths, path) {
            relayout_node_along_paths(child, rect, path, dirty_paths, stats);
        }
        path.pop();
    }

    if affects_node || width_changed {
        node.measured_height = remeasure_node_height(node, bounds.width).max(1);
    }
}

fn child_rects<Message>(element: &Element<Message>, bounds: Rect) -> Vec<Rect> {
    let spec = element.child_layout_spec(bounds);
    match spec.kind {
        ChildLayoutKind::Fill => fill_children(element.children.len(), spec.bounds),
        ChildLayoutKind::Shell => {
            shell_child_rects_with_heights(spec.bounds, element.children.len(), |index, width| {
                measured_height(
                    &element.children[index],
                    child_width(&element.children[index], width),
                )
            })
        }
        ChildLayoutKind::Stack { direction, gap } => stack_child_rects(
            element.children.len(),
            spec.bounds,
            direction,
            gap,
            |index| main_length(&element.children[index], direction),
            |index, child_bounds, child_direction| {
                auto_main_length(&element.children[index], child_bounds, child_direction)
            },
            |index, child_bounds, child_direction| {
                cross_size_for(&element.children[index], child_bounds, child_direction)
            },
        ),
    }
}

fn child_rects_for_node<Message>(node: &Node<Message>, bounds: Rect) -> Vec<Rect> {
    let spec = node.child_layout_spec(bounds);
    match spec.kind {
        ChildLayoutKind::Fill => fill_children(node.children.len(), spec.bounds),
        ChildLayoutKind::Shell => {
            shell_child_rects_with_heights(spec.bounds, node.children.len(), |index, width| {
                measured_node_height_cached(
                    &node.children[index],
                    child_node_width(&node.children[index], width),
                )
            })
        }
        ChildLayoutKind::Stack { direction, gap } => stack_child_rects(
            node.children.len(),
            spec.bounds,
            direction,
            gap,
            |index| main_length_for_layout(node.children[index].element.layout, direction),
            |index, child_bounds, child_direction| {
                auto_main_length_node(&node.children[index], child_bounds, child_direction)
            },
            |index, child_bounds, child_direction| {
                cross_size_for_layout(
                    node.children[index].element.layout,
                    child_bounds,
                    child_direction,
                )
            },
        ),
    }
}

fn fill_children(count: usize, bounds: Rect) -> Vec<Rect> {
    (0..count).map(|_| bounds).collect()
}

fn shell_child_rects_with_heights(
    bounds: Rect,
    child_count: usize,
    mut child_height_at: impl FnMut(usize, u16) -> u16,
) -> Vec<Rect> {
    debug_assert!(
        child_count <= 3,
        "Shell takes at most 3 children: header / body / footer"
    );

    match child_count {
        0 => Vec::new(),
        1 => vec![bounds],
        2 => {
            let header_height = child_height_at(0, bounds.width).min(bounds.height);
            vec![
                Rect::new(bounds.x, bounds.y, bounds.width, header_height),
                Rect::new(
                    bounds.x,
                    bounds.y.saturating_add(header_height),
                    bounds.width,
                    bounds.height.saturating_sub(header_height),
                ),
            ]
        }
        _ => {
            let header_height = child_height_at(0, bounds.width).min(bounds.height);
            let footer_height =
                child_height_at(2, bounds.width).min(bounds.height.saturating_sub(header_height));
            let body_y = bounds.y.saturating_add(header_height);
            let body_height = bounds
                .height
                .saturating_sub(header_height.saturating_add(footer_height));
            let footer_y = bounds
                .y
                .saturating_add(bounds.height.saturating_sub(footer_height));

            vec![
                Rect::new(bounds.x, bounds.y, bounds.width, header_height),
                Rect::new(bounds.x, body_y, bounds.width, body_height),
                Rect::new(bounds.x, footer_y, bounds.width, footer_height),
            ]
        }
    }
}

fn stack_child_rects(
    len: usize,
    bounds: Rect,
    direction: Direction,
    gap: u16,
    mut main_length_at: impl FnMut(usize) -> Length,
    mut auto_main_length_at: impl FnMut(usize, Rect, Direction) -> u16,
    mut cross_size_at: impl FnMut(usize, Rect, Direction) -> u16,
) -> Vec<Rect> {
    if len == 0 {
        return Vec::new();
    }

    let gap_total = gap.saturating_mul(len.saturating_sub(1) as u16);
    let main_available = main_size(bounds, direction).saturating_sub(gap_total);

    let mut reserved = 0u16;
    let mut fill_count = 0u16;

    for index in 0..len {
        match main_length_at(index) {
            Length::Fixed(size) => reserved = reserved.saturating_add(size),
            Length::Auto => {
                reserved = reserved.saturating_add(auto_main_length_at(index, bounds, direction))
            }
            Length::Fill => fill_count = fill_count.saturating_add(1),
        }
    }

    let fill_available = main_available.saturating_sub(reserved);
    let fill_base = if fill_count == 0 {
        0
    } else {
        fill_available / fill_count
    };
    let fill_remainder = if fill_count == 0 {
        0
    } else {
        fill_available % fill_count
    };

    let mut cursor_x = bounds.x;
    let mut cursor_y = bounds.y;
    let mut remaining = main_size(bounds, direction);
    let mut assigned_fill = 0u16;
    let mut rects = Vec::with_capacity(len);

    for index in 0..len {
        let wants_gap = !rects.is_empty();
        if wants_gap {
            let gap_size = gap.min(remaining);
            advance_cursor(&mut cursor_x, &mut cursor_y, direction, gap_size);
            remaining = remaining.saturating_sub(gap_size);
        }

        let planned_main = match main_length_at(index) {
            Length::Fixed(size) => size,
            Length::Auto => auto_main_length_at(index, bounds, direction),
            Length::Fill => {
                let extra = u16::from(assigned_fill < fill_remainder);
                assigned_fill = assigned_fill.saturating_add(1);
                fill_base.saturating_add(extra)
            }
        };
        let actual_main = planned_main.min(remaining);
        let cross = cross_size_at(index, bounds, direction);

        rects.push(match direction {
            Direction::Column => Rect::new(cursor_x, cursor_y, cross, actual_main),
            Direction::Row => Rect::new(cursor_x, cursor_y, actual_main, cross),
        });

        advance_cursor(&mut cursor_x, &mut cursor_y, direction, actual_main);
        remaining = remaining.saturating_sub(actual_main);
    }

    rects
}

fn main_length<Message>(element: &Element<Message>, direction: Direction) -> Length {
    main_length_for_layout(element.layout, direction)
}

fn main_length_for_layout(layout: ansiq_core::Layout, direction: Direction) -> Length {
    match direction {
        Direction::Column => layout.height,
        Direction::Row => layout.width,
    }
}

fn cross_length_for_layout(layout: ansiq_core::Layout, direction: Direction) -> Length {
    match direction {
        Direction::Column => layout.width,
        Direction::Row => layout.height,
    }
}

fn main_size(bounds: Rect, direction: Direction) -> u16 {
    match direction {
        Direction::Column => bounds.height,
        Direction::Row => bounds.width,
    }
}

fn cross_size(bounds: Rect, direction: Direction) -> u16 {
    match direction {
        Direction::Column => bounds.width,
        Direction::Row => bounds.height,
    }
}

fn cross_size_for<Message>(element: &Element<Message>, bounds: Rect, direction: Direction) -> u16 {
    cross_size_for_layout(element.layout, bounds, direction)
}

fn cross_size_for_layout(layout: ansiq_core::Layout, bounds: Rect, direction: Direction) -> u16 {
    match cross_length_for_layout(layout, direction) {
        Length::Fixed(size) => size.min(cross_size(bounds, direction)),
        Length::Auto | Length::Fill => cross_size(bounds, direction),
    }
}

fn advance_cursor(cursor_x: &mut u16, cursor_y: &mut u16, direction: Direction, amount: u16) {
    match direction {
        Direction::Column => *cursor_y = cursor_y.saturating_add(amount),
        Direction::Row => *cursor_x = cursor_x.saturating_add(amount),
    }
}

fn auto_main_length<Message>(
    element: &Element<Message>,
    bounds: Rect,
    direction: Direction,
) -> u16 {
    match direction {
        Direction::Column => measured_height(element, cross_size(bounds, direction)),
        Direction::Row => measured_width(element).min(main_size(bounds, direction)),
    }
}

fn auto_main_length_node<Message>(node: &Node<Message>, bounds: Rect, direction: Direction) -> u16 {
    match direction {
        Direction::Column => measured_node_height_cached(node, cross_size(bounds, direction)),
        Direction::Row => measured_node_width(node).min(main_size(bounds, direction)),
    }
}

fn measured_height<Message>(element: &Element<Message>, width: u16) -> u16 {
    match element.layout.height {
        Length::Fixed(height) => height,
        Length::Auto | Length::Fill => intrinsic_measured_height(element, width),
    }
}

fn measured_node_height_cached<Message>(node: &Node<Message>, width: u16) -> u16 {
    if width == node.rect.width && node.measured_height > 0 {
        return node.measured_height;
    }

    remeasure_node_height(node, width)
}

fn remeasure_node_height<Message>(node: &Node<Message>, width: u16) -> u16 {
    match node.element.layout.height {
        Length::Fixed(height) => height,
        Length::Auto | Length::Fill => intrinsic_node_height(node, width),
    }
}

fn intrinsic_measured_height<Message>(element: &Element<Message>, width: u16) -> u16 {
    let child_width_base = element
        .child_layout_spec(Rect::new(0, 0, width, u16::MAX))
        .bounds
        .width;
    let child_heights = element
        .children
        .iter()
        .map(|child| measured_height(child, child_width(child, child_width_base)))
        .collect::<Vec<_>>();
    element.intrinsic_height(width, &child_heights)
}

fn measured_width<Message>(element: &Element<Message>) -> u16 {
    match element.layout.width {
        Length::Fixed(width) => width,
        Length::Auto | Length::Fill => intrinsic_measured_width(element),
    }
}

fn measured_node_width<Message>(node: &Node<Message>) -> u16 {
    match node.element.layout.width {
        Length::Fixed(width) => width,
        Length::Auto | Length::Fill => intrinsic_node_width(node),
    }
}

fn intrinsic_measured_width<Message>(element: &Element<Message>) -> u16 {
    let child_widths = element
        .children
        .iter()
        .map(measured_width)
        .collect::<Vec<_>>();
    element.intrinsic_width(&child_widths)
}

fn intrinsic_node_width<Message>(node: &Node<Message>) -> u16 {
    let child_widths = node
        .children
        .iter()
        .map(measured_node_width)
        .collect::<Vec<_>>();
    node.element.intrinsic_width(&child_widths)
}

fn intrinsic_node_height<Message>(node: &Node<Message>, width: u16) -> u16 {
    let child_width_base = node
        .child_layout_spec(Rect::new(0, 0, width, u16::MAX))
        .bounds
        .width;
    let child_heights = node
        .children
        .iter()
        .map(|child| measured_node_height_cached(child, child_node_width(child, child_width_base)))
        .collect::<Vec<_>>();
    node.element.intrinsic_height(width, &child_heights)
}

fn child_width<Message>(element: &Element<Message>, available_width: u16) -> u16 {
    match element.layout.width {
        Length::Fixed(width) => width.min(available_width),
        Length::Auto | Length::Fill => available_width,
    }
}

fn child_node_width<Message>(node: &Node<Message>, available_width: u16) -> u16 {
    match node.element.layout.width {
        Length::Fixed(width) => width.min(available_width),
        Length::Auto | Length::Fill => available_width,
    }
}

fn path_affects_node(dirty_paths: &[Vec<usize>], path: &[usize]) -> bool {
    dirty_paths
        .iter()
        .any(|dirty_path| dirty_path.starts_with(path))
}

fn path_is_dirty_target(dirty_paths: &[Vec<usize>], path: &[usize]) -> bool {
    dirty_paths.iter().any(|dirty_path| dirty_path == path)
}

fn normalize_dirty_paths(dirty_paths: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let mut normalized = dirty_paths.to_vec();
    normalized.sort();
    normalized.dedup();

    let mut compressed = Vec::with_capacity(normalized.len());
    for path in normalized {
        if compressed
            .iter()
            .any(|existing: &Vec<usize>| path.starts_with(existing))
        {
            continue;
        }
        compressed.push(path);
    }

    compressed
}

fn push_invalidated_region(regions: &mut Vec<Rect>, rect: Rect) {
    if rect.is_empty() {
        return;
    }

    if regions.iter().any(|existing| existing.contains(rect)) {
        return;
    }

    let mut merged = rect;
    let mut index = 0;
    while index < regions.len() {
        let existing = regions[index];
        if merged.contains(existing) || merged.can_merge_rect(existing) {
            merged = merged.union(existing);
            regions.remove(index);
            continue;
        }
        index += 1;
    }

    regions.push(merged);
}

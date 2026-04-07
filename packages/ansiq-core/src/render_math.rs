use crate::{Constraint, Flex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TitleGroupPositions {
    pub left_x: Option<u16>,
    pub center_x: Option<u16>,
    pub right_x: Option<u16>,
}

pub fn title_group_positions(
    area_width: u16,
    left_width: u16,
    center_width: u16,
    right_width: u16,
) -> TitleGroupPositions {
    if area_width == 0 {
        return TitleGroupPositions {
            left_x: None,
            center_x: None,
            right_x: None,
        };
    }

    let left_x = (left_width > 0).then_some(0u16);
    let right_x = (right_width > 0).then(|| area_width.saturating_sub(right_width.min(area_width)));

    let center_x = if center_width > 0 {
        let center_width = center_width.min(area_width);
        let center_x = area_width.saturating_sub(center_width) / 2;
        let center_end = center_x.saturating_add(center_width);
        let left_end = left_x
            .map(|x| x.saturating_add(left_width.min(area_width)))
            .unwrap_or(0);
        let right_start = right_x.unwrap_or(area_width);
        (center_x >= left_end && center_end <= right_start).then_some(center_x)
    } else {
        None
    };

    TitleGroupPositions {
        left_x,
        center_x,
        right_x,
    }
}

pub fn table_span_width(
    column_widths: &[u16],
    column_positions: &[u16],
    table_width: u16,
    start: usize,
    span: usize,
) -> u16 {
    if start >= column_positions.len() {
        return 0;
    }

    let start_x = column_positions[start];
    let last = start
        .saturating_add(span.saturating_sub(1))
        .min(column_positions.len().saturating_sub(1));
    let end_x = column_positions[last]
        .saturating_add(column_widths.get(last).copied().unwrap_or_default())
        .min(table_width);

    end_x.saturating_sub(start_x).max(
        column_widths
            .get(start)
            .copied()
            .unwrap_or_default()
            .min(table_width.saturating_sub(start_x)),
    )
}

pub fn table_column_layout(
    total_width: u16,
    columns: usize,
    widths: &[Constraint],
    column_spacing: u16,
    flex: Flex,
) -> (Vec<u16>, Vec<u16>) {
    if columns == 0 {
        return (Vec::new(), Vec::new());
    }

    let mut effective_spacing = column_spacing;
    let separator_width = columns.saturating_sub(1) as u16 * effective_spacing;
    let content_width = total_width.saturating_sub(separator_width);
    if widths.is_empty() {
        let base = content_width / columns as u16;
        let remainder = content_width % columns as u16;
        let widths: Vec<u16> = (0..columns)
            .map(|index| base.saturating_add(u16::from(index < remainder as usize)))
            .collect();
        let positions =
            table_column_positions(columns, &widths, effective_spacing, total_width, flex);
        return (widths, positions);
    }

    let mut resolved = vec![0u16; columns];
    let mut fixed_total = 0u16;
    let mut fill_columns = Vec::new();
    let mut max_columns = Vec::new();
    let mut min_columns = Vec::new();

    for index in 0..columns {
        let constraint = widths.get(index).copied().unwrap_or(Constraint::Fill(1));
        match constraint {
            Constraint::Length(value) => {
                resolved[index] = value;
                fixed_total = fixed_total.saturating_add(value);
            }
            Constraint::Percentage(percent) => {
                let value = ((u32::from(content_width) * u32::from(percent.min(100))) / 100) as u16;
                resolved[index] = value;
                fixed_total = fixed_total.saturating_add(value);
            }
            Constraint::Min(value) => {
                resolved[index] = value;
                fixed_total = fixed_total.saturating_add(value);
                min_columns.push(index);
            }
            Constraint::Max(value) => max_columns.push((index, value)),
            Constraint::Fill(weight) => fill_columns.push((index, weight.max(1))),
        }
    }

    let mut remaining = content_width.saturating_sub(fixed_total);
    if !fill_columns.is_empty() {
        let total_weight: u16 = fill_columns.iter().map(|(_, weight)| *weight).sum();
        let mut assigned = 0u16;
        for (position, (index, weight)) in fill_columns.iter().enumerate() {
            let share = if position == fill_columns.len() - 1 {
                remaining.saturating_sub(assigned)
            } else {
                ((u32::from(remaining) * u32::from(*weight)) / u32::from(total_weight.max(1)))
                    as u16
            };
            resolved[*index] = share;
            assigned = assigned.saturating_add(share);
        }
        fit_table_columns_to_width(total_width, &mut resolved, &mut effective_spacing);
        let positions =
            table_column_positions(columns, &resolved, effective_spacing, total_width, flex);
        return (resolved, positions);
    }

    if !max_columns.is_empty() && remaining > 0 {
        for (index, max_width) in max_columns {
            let share = remaining.min(max_width);
            resolved[index] = share;
            remaining = remaining.saturating_sub(share);
            if remaining == 0 {
                break;
            }
        }
    }

    if matches!(flex, Flex::Legacy) {
        let stretch_targets: Vec<usize> = if !min_columns.is_empty() {
            min_columns
        } else {
            (0..columns).collect()
        };
        if remaining > 0 && !stretch_targets.is_empty() {
            let base = remaining / stretch_targets.len() as u16;
            let remainder = remaining % stretch_targets.len() as u16;
            for (position, index) in stretch_targets.into_iter().enumerate() {
                resolved[index] = resolved[index]
                    .saturating_add(base)
                    .saturating_add(u16::from(position < remainder as usize));
            }
        }
        fit_table_columns_to_width(total_width, &mut resolved, &mut effective_spacing);
        let positions =
            table_column_positions(columns, &resolved, effective_spacing, total_width, flex);
        return (resolved, positions);
    }

    if remaining > 0 && !min_columns.is_empty() {
        let mut stretch_targets = min_columns;
        stretch_targets.sort_by_key(|index| resolved[*index]);

        let mut active = 1usize;
        while active < stretch_targets.len() && remaining > 0 {
            let current = resolved[stretch_targets[active - 1]];
            let next = resolved[stretch_targets[active]];
            let delta = next.saturating_sub(current);
            if delta == 0 {
                active += 1;
                continue;
            }

            let needed = delta.saturating_mul(active as u16);
            if remaining < needed {
                break;
            }

            for index in &stretch_targets[..active] {
                resolved[*index] = resolved[*index].saturating_add(delta);
            }
            remaining = remaining.saturating_sub(needed);
            active += 1;
        }

        let base = remaining / active as u16;
        let remainder = remaining % active as u16;
        for (position, index) in stretch_targets[..active].iter().enumerate() {
            resolved[*index] = resolved[*index]
                .saturating_add(base)
                .saturating_add(u16::from(position < remainder as usize));
        }
    }

    fit_table_columns_to_width(total_width, &mut resolved, &mut effective_spacing);
    let positions =
        table_column_positions(columns, &resolved, effective_spacing, total_width, flex);

    (resolved, positions)
}

fn fit_table_columns_to_width(total_width: u16, widths: &mut [u16], column_spacing: &mut u16) {
    if widths.is_empty() {
        *column_spacing = 0;
        return;
    }

    let gaps = widths.len().saturating_sub(1) as u16;
    let preferred_width = widths.iter().copied().fold(0u16, u16::saturating_add);
    let preferred_total = preferred_width.saturating_add(gaps.saturating_mul(*column_spacing));
    if preferred_total <= total_width {
        return;
    }

    if gaps > 0 {
        let overflow = preferred_total.saturating_sub(total_width);
        let max_spacing_reduction = gaps.saturating_mul(*column_spacing);
        let spacing_reduction = overflow.min(max_spacing_reduction);
        let remaining_spacing = max_spacing_reduction.saturating_sub(spacing_reduction);
        *column_spacing = remaining_spacing / gaps;
    }

    let available_for_columns = total_width.saturating_sub(gaps.saturating_mul(*column_spacing));
    let width_sum = widths.iter().copied().fold(0u16, u16::saturating_add);
    if width_sum <= available_for_columns {
        return;
    }

    if available_for_columns == 0 {
        widths.fill(0);
        return;
    }

    let mut reassigned = vec![0u16; widths.len()];
    let mut assigned = 0u16;
    for (index, width) in widths.iter().copied().enumerate() {
        let share = ((u32::from(available_for_columns) * u32::from(width))
            / u32::from(width_sum.max(1))) as u16;
        reassigned[index] = share;
        assigned = assigned.saturating_add(share);
    }
    let mut remainder = available_for_columns.saturating_sub(assigned);
    for width in &mut reassigned {
        if remainder == 0 {
            break;
        }
        *width = width.saturating_add(1);
        remainder = remainder.saturating_sub(1);
    }

    widths.copy_from_slice(&reassigned);
}

fn table_column_positions(
    columns: usize,
    widths: &[u16],
    column_spacing: u16,
    total_width: u16,
    flex: Flex,
) -> Vec<u16> {
    if columns == 0 {
        return Vec::new();
    }

    let separator_width = columns.saturating_sub(1) as u16 * column_spacing;
    let used_width = widths
        .iter()
        .copied()
        .fold(0u16, u16::saturating_add)
        .saturating_add(separator_width)
        .min(total_width);
    let extra = total_width.saturating_sub(used_width);

    let (leading, between_extra) = match flex {
        Flex::End => (extra, vec![0; columns.saturating_sub(1)]),
        Flex::Center => (extra / 2, vec![0; columns.saturating_sub(1)]),
        Flex::SpaceBetween if columns > 1 => {
            let gaps = columns - 1;
            let base = extra / gaps as u16;
            let remainder = extra % gaps as u16;
            let between = (0..gaps)
                .map(|index| base.saturating_add(u16::from(index < remainder as usize)))
                .collect();
            (0, between)
        }
        Flex::SpaceAround if columns > 0 => distributed_space_around_gaps(extra, columns),
        Flex::SpaceEvenly if columns > 0 => distributed_space_evenly_gaps(extra, columns),
        _ => (0, vec![0; columns.saturating_sub(1)]),
    };

    let mut positions = Vec::with_capacity(columns);
    let mut x = leading;
    for index in 0..columns {
        positions.push(x);
        x = x.saturating_add(widths.get(index).copied().unwrap_or(0));
        if index + 1 < columns {
            x = x
                .saturating_add(column_spacing)
                .saturating_add(between_extra.get(index).copied().unwrap_or(0));
        }
    }
    positions
}

fn distributed_space_evenly_gaps(extra: u16, columns: usize) -> (u16, Vec<u16>) {
    if columns == 0 {
        return (0, Vec::new());
    }

    let gap_count = columns + 1;
    let base = extra / gap_count as u16;
    let remainder = extra % gap_count as u16;
    let mut gaps = Vec::with_capacity(gap_count);
    for index in 0..gap_count {
        gaps.push(base.saturating_add(u16::from(index < remainder as usize)));
    }

    let leading = gaps.first().copied().unwrap_or(0);
    let between = if gaps.len() > 2 {
        gaps[1..gaps.len() - 1].to_vec()
    } else {
        Vec::new()
    };
    (leading, between)
}

fn distributed_space_around_gaps(extra: u16, columns: usize) -> (u16, Vec<u16>) {
    if columns == 0 {
        return (0, Vec::new());
    }

    let edge_base = extra / (columns.saturating_mul(2) as u16);
    let between_base = edge_base.saturating_mul(2);
    let mut gaps = Vec::with_capacity(columns + 1);
    gaps.push(edge_base);
    for _ in 1..columns {
        gaps.push(between_base);
    }
    gaps.push(edge_base);

    let allocated = edge_base
        .saturating_mul(2)
        .saturating_add(between_base.saturating_mul(columns.saturating_sub(1) as u16));
    let mut remainder = extra.saturating_sub(allocated);

    let mut gap_order = Vec::with_capacity(gaps.len());
    gap_order.extend(1..columns);
    gap_order.push(0);
    gap_order.push(columns);

    while remainder > 0 {
        for index in &gap_order {
            if remainder == 0 {
                break;
            }
            gaps[*index] = gaps[*index].saturating_add(1);
            remainder = remainder.saturating_sub(1);
        }
    }

    let leading = gaps.first().copied().unwrap_or(0);
    let between = if gaps.len() > 2 {
        gaps[1..gaps.len() - 1].to_vec()
    } else {
        Vec::new()
    };
    (leading, between)
}

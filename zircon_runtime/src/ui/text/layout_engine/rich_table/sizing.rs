use crate::core::framework::render::RichTableColumn;

#[derive(Clone, Copy, Debug)]
pub(super) struct PreferredColumnExtent {
    pub column: usize,
    pub column_span: usize,
    pub extent: f32,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RowExtentConstraint {
    pub row: usize,
    pub row_span: usize,
    pub extent: f32,
}

pub(super) fn resolve_column_extents(
    columns: &[RichTableColumn],
    preferred_cells: &[PreferredColumnExtent],
    available_track_extent: f32,
    gap: f32,
    minimum: f32,
) -> Vec<f32> {
    let policies = if columns.is_empty() {
        vec![RichTableColumn::default()]
    } else {
        columns.to_vec()
    };
    let mut widths = vec![sanitize_non_negative(minimum); policies.len()];

    for cell in preferred_cells.iter().filter(|cell| cell.column_span == 1) {
        if let Some(width) = widths.get_mut(cell.column) {
            *width = width.max(sanitize_non_negative(cell.extent));
        }
    }

    let mut spanning_cells = preferred_cells
        .iter()
        .filter(|cell| cell.column_span > 1)
        .collect::<Vec<_>>();
    spanning_cells.sort_by_key(|cell| cell.column_span);
    for cell in spanning_cells {
        let end = cell
            .column
            .saturating_add(cell.column_span)
            .min(widths.len());
        if cell.column >= end {
            continue;
        }
        let track_width = widths[cell.column..end].iter().sum::<f32>()
            + sanitize_non_negative(gap) * end.saturating_sub(cell.column + 1) as f32;
        let deficit = (sanitize_non_negative(cell.extent) - track_width).max(0.0);
        distribute_column_deficit(
            &mut widths[cell.column..end],
            &policies[cell.column..end],
            deficit,
        );
    }

    fit_columns_to_available_width(
        &mut widths,
        &policies,
        sanitize_non_negative(available_track_extent),
        sanitize_non_negative(minimum),
    );
    widths
}

pub(super) fn resolve_row_extents(
    row_count: usize,
    cells: &[RowExtentConstraint],
    minimum: f32,
) -> Vec<f32> {
    let mut heights = vec![sanitize_non_negative(minimum); row_count];
    for cell in cells.iter().filter(|cell| cell.row_span == 1) {
        if let Some(height) = heights.get_mut(cell.row) {
            *height = height.max(sanitize_non_negative(cell.extent));
        }
    }

    let mut spanning_cells = cells
        .iter()
        .filter(|cell| cell.row_span > 1)
        .collect::<Vec<_>>();
    spanning_cells.sort_by_key(|cell| cell.row_span);
    for cell in spanning_cells {
        let end = cell.row.saturating_add(cell.row_span).min(heights.len());
        if cell.row >= end {
            continue;
        }
        let current = heights[cell.row..end].iter().sum::<f32>();
        let deficit = (sanitize_non_negative(cell.extent) - current).max(0.0);
        let share = deficit / end.saturating_sub(cell.row).max(1) as f32;
        for height in &mut heights[cell.row..end] {
            *height += share;
        }
    }
    heights
}

fn distribute_column_deficit(widths: &mut [f32], policies: &[RichTableColumn], deficit: f32) {
    if deficit <= 0.0 || widths.is_empty() {
        return;
    }
    let expand_ratio = policies
        .iter()
        .filter(|column| column.expand)
        .map(|column| u32::from(column.expand_ratio))
        .sum::<u32>();
    if expand_ratio > 0 {
        for (width, column) in widths.iter_mut().zip(policies) {
            if column.expand {
                *width += deficit * f32::from(column.expand_ratio) / expand_ratio as f32;
            }
        }
    } else {
        let share = deficit / widths.len() as f32;
        for width in widths {
            *width += share;
        }
    }
}

fn fit_columns_to_available_width(
    widths: &mut [f32],
    policies: &[RichTableColumn],
    available_width: f32,
    minimum: f32,
) {
    let preferred_total = widths.iter().sum::<f32>();
    if preferred_total > available_width && preferred_total > 0.0 {
        let fixed_total = widths
            .iter()
            .zip(policies)
            .filter(|(_, column)| !column.shrink)
            .map(|(width, _)| *width)
            .sum::<f32>();
        let shrink_total = widths
            .iter()
            .zip(policies)
            .filter(|(_, column)| column.shrink)
            .map(|(width, _)| *width)
            .sum::<f32>();
        let shrink_budget = (available_width - fixed_total).max(0.0);
        if shrink_total > 0.0 {
            for (width, column) in widths.iter_mut().zip(policies) {
                if column.shrink {
                    *width = (*width * shrink_budget / shrink_total).max(minimum);
                }
            }
        }
    } else if preferred_total < available_width {
        let expand_ratio = policies
            .iter()
            .filter(|column| column.expand)
            .map(|column| u32::from(column.expand_ratio))
            .sum::<u32>();
        if expand_ratio > 0 {
            let extra = available_width - preferred_total;
            for (width, column) in widths.iter_mut().zip(policies) {
                if column.expand {
                    *width += extra * f32::from(column.expand_ratio) / expand_ratio as f32;
                }
            }
        }
    }
}

fn sanitize_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

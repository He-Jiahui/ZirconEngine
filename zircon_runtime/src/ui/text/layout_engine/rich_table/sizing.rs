use crate::text::{
    RichTableColumn, TextLayoutAxisConstraint, TextLayoutGeometryBudget,
    TextLayoutGeometryViolation,
};

const SHRINK_SCALE_SEARCH_STEPS: usize = 24;

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
    available_track_extent: TextLayoutAxisConstraint,
    gap: f32,
    minimum: f32,
    budget: TextLayoutGeometryBudget,
) -> Result<Vec<f32>, TextLayoutGeometryViolation> {
    let gap = budget.admit_axis_extent(gap)?;
    let minimum = budget.admit_axis_extent(minimum)?;
    let policies = if columns.is_empty() {
        vec![RichTableColumn::default()]
    } else {
        columns.to_vec()
    };
    let mut widths = vec![minimum; policies.len()];

    for cell in preferred_cells.iter().filter(|cell| cell.column_span == 1) {
        let cell_extent = budget.admit_axis_extent(cell.extent)?;
        if let Some(width) = widths.get_mut(cell.column) {
            *width = width.max(cell_extent);
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
        let track_extent =
            checked_sum_accumulated(widths[cell.column..end].iter().copied(), budget)?;
        let internal_gap_extent =
            budget.checked_scale_accumulated(gap, end.saturating_sub(cell.column + 1))?;
        let track_extent = budget.checked_add_accumulated(track_extent, internal_gap_extent)?;
        let cell_extent = budget.admit_axis_extent(cell.extent)?;
        let deficit = (cell_extent - track_extent).max(0.0);
        distribute_column_deficit(
            &mut widths[cell.column..end],
            &policies[cell.column..end],
            deficit,
            budget,
        )?;
    }

    fit_columns_to_available_width(
        &mut widths,
        &policies,
        available_track_extent,
        minimum,
        budget,
    )?;
    for width in &widths {
        budget.admit_axis_extent(*width)?;
    }
    Ok(widths)
}

pub(super) fn resolve_row_extents(
    row_count: usize,
    cells: &[RowExtentConstraint],
    minimum: f32,
    budget: TextLayoutGeometryBudget,
) -> Result<Vec<f32>, TextLayoutGeometryViolation> {
    let minimum = budget.admit_axis_extent(minimum)?;
    let mut heights = vec![minimum; row_count];
    for cell in cells.iter().filter(|cell| cell.row_span == 1) {
        let cell_extent = budget.admit_axis_extent(cell.extent)?;
        if let Some(height) = heights.get_mut(cell.row) {
            *height = height.max(cell_extent);
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
        let current = checked_sum_accumulated(heights[cell.row..end].iter().copied(), budget)?;
        let cell_extent = budget.admit_axis_extent(cell.extent)?;
        let deficit = (cell_extent - current).max(0.0);
        let share = deficit / end.saturating_sub(cell.row).max(1) as f32;
        budget.admit_axis_extent(share)?;
        for height in &mut heights[cell.row..end] {
            *height = budget
                .checked_add_accumulated(*height, share)
                .and_then(|height| budget.admit_axis_extent(height))?;
        }
    }
    Ok(heights)
}

fn distribute_column_deficit(
    widths: &mut [f32],
    policies: &[RichTableColumn],
    deficit: f32,
    budget: TextLayoutGeometryBudget,
) -> Result<(), TextLayoutGeometryViolation> {
    budget.admit_accumulated_extent(deficit)?;
    if deficit <= 0.0 || widths.is_empty() {
        return Ok(());
    }
    let expand_ratio = policies
        .iter()
        .filter(|column| column.expand)
        .map(|column| u32::from(column.expand_ratio))
        .sum::<u32>();
    if expand_ratio > 0 {
        for (width, column) in widths.iter_mut().zip(policies) {
            if column.expand {
                let share = deficit * f32::from(column.expand_ratio) / expand_ratio as f32;
                *width = budget
                    .checked_add_accumulated(*width, share)
                    .and_then(|width| budget.admit_axis_extent(width))?;
            }
        }
    } else {
        let share = deficit / widths.len() as f32;
        budget.admit_axis_extent(share)?;
        for width in widths {
            *width = budget
                .checked_add_accumulated(*width, share)
                .and_then(|width| budget.admit_axis_extent(width))?;
        }
    }
    Ok(())
}

fn fit_columns_to_available_width(
    widths: &mut [f32],
    policies: &[RichTableColumn],
    available_width: TextLayoutAxisConstraint,
    minimum: f32,
    budget: TextLayoutGeometryBudget,
) -> Result<(), TextLayoutGeometryViolation> {
    let Some(available_width) = available_width.bounded_extent() else {
        return Ok(());
    };
    let preferred_total = checked_sum_accumulated(widths.iter().copied(), budget)?;
    if preferred_total > available_width && preferred_total > 0.0 {
        let fixed_total = checked_sum_accumulated(
            widths
                .iter()
                .zip(policies)
                .filter(|(_, column)| !column.shrink)
                .map(|(width, _)| *width),
            budget,
        )?;
        let shrink_budget = (available_width - fixed_total).max(0.0);
        shrink_columns_to_budget(widths, policies, shrink_budget, minimum, budget)?;
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
                    let share = extra * f32::from(column.expand_ratio) / expand_ratio as f32;
                    *width = budget
                        .checked_add_accumulated(*width, share)
                        .and_then(|width| budget.admit_axis_extent(width))?;
                }
            }
        }
    }
    Ok(())
}

fn shrink_columns_to_budget(
    widths: &mut [f32],
    policies: &[RichTableColumn],
    shrink_budget: f32,
    minimum: f32,
    budget: TextLayoutGeometryBudget,
) -> Result<(), TextLayoutGeometryViolation> {
    let shrinkable_count = policies.iter().filter(|column| column.shrink).count();
    if shrinkable_count == 0 {
        return Ok(());
    }

    let minimum_total = budget.checked_scale_accumulated(minimum, shrinkable_count)?;
    if shrink_budget <= minimum_total {
        for (width, column) in widths.iter_mut().zip(policies) {
            if column.shrink {
                *width = minimum;
            }
        }
        return Ok(());
    }

    let preferred_shrink_total = checked_sum_accumulated(
        widths
            .iter()
            .zip(policies)
            .filter(|(_, column)| column.shrink)
            .map(|(width, _)| *width),
        budget,
    )?;
    if preferred_shrink_total <= shrink_budget {
        return Ok(());
    }

    let mut lower_scale = 0.0_f32;
    let mut upper_scale = 1.0_f32;
    for _ in 0..SHRINK_SCALE_SEARCH_STEPS {
        let scale = (lower_scale + upper_scale) * 0.5;
        let resolved_shrink_total = checked_sum_accumulated(
            widths
                .iter()
                .zip(policies)
                .filter(|(_, column)| column.shrink)
                .map(|(width, _)| (*width * scale).max(minimum)),
            budget,
        )?;
        if resolved_shrink_total > shrink_budget {
            upper_scale = scale;
        } else {
            lower_scale = scale;
        }
    }

    for (width, column) in widths.iter_mut().zip(policies) {
        if column.shrink {
            *width = budget.admit_axis_extent((*width * lower_scale).max(minimum))?;
        }
    }
    Ok(())
}

fn checked_sum_accumulated(
    mut values: impl Iterator<Item = f32>,
    budget: TextLayoutGeometryBudget,
) -> Result<f32, TextLayoutGeometryViolation> {
    values.try_fold(0.0, |total, value| {
        let value = budget.admit_axis_extent(value)?;
        budget.checked_add_accumulated(total, value)
    })
}

#[cfg(test)]
mod tests {
    use crate::text::{RichTableColumn, TextLayoutAxisConstraint, TextLayoutGeometryBudget};

    use super::{fit_columns_to_available_width, resolve_column_extents};

    fn shrinkable() -> RichTableColumn {
        RichTableColumn::default()
    }

    fn fixed() -> RichTableColumn {
        RichTableColumn {
            shrink: false,
            ..RichTableColumn::default()
        }
    }

    fn budget() -> TextLayoutGeometryBudget {
        TextLayoutGeometryBudget::new(1_000.0, 4_000.0).expect("valid test budget")
    }

    #[test]
    fn shrink_budget_accounts_for_columns_that_reach_the_minimum() {
        let mut widths = [20.0, 190.0];
        fit_columns_to_available_width(
            &mut widths,
            &[shrinkable(), shrinkable()],
            TextLayoutAxisConstraint::Bounded(100.0),
            20.0,
            budget(),
        )
        .expect("valid geometry");

        assert!((widths[0] - 20.0).abs() < 0.001);
        assert!((widths[1] - 80.0).abs() < 0.001);
        assert!((widths.iter().sum::<f32>() - 100.0).abs() < 0.001);
    }

    #[test]
    fn shrink_budget_excludes_fixed_columns_before_solving_lower_bounds() {
        let mut widths = [70.0, 40.0, 160.0];
        fit_columns_to_available_width(
            &mut widths,
            &[fixed(), shrinkable(), shrinkable()],
            TextLayoutAxisConstraint::Bounded(150.0),
            20.0,
            budget(),
        )
        .expect("valid geometry");

        assert!((widths[0] - 70.0).abs() < 0.001);
        assert!((widths[1] - 20.0).abs() < 0.001);
        assert!((widths[2] - 60.0).abs() < 0.001);
        assert!((widths.iter().sum::<f32>() - 150.0).abs() < 0.001);
    }

    #[test]
    fn minimum_width_wins_when_the_available_budget_is_infeasible() {
        let mut widths = [60.0, 80.0, 120.0];
        fit_columns_to_available_width(
            &mut widths,
            &[fixed(), shrinkable(), shrinkable()],
            TextLayoutAxisConstraint::Bounded(80.0),
            20.0,
            budget(),
        )
        .expect("valid geometry");

        assert_eq!(widths, [60.0, 20.0, 20.0]);
    }

    #[test]
    fn unbounded_available_width_retains_natural_column_extents() {
        let widths = resolve_column_extents(
            &[shrinkable(), shrinkable()],
            &[],
            TextLayoutAxisConstraint::Unbounded,
            4.0,
            20.0,
            budget(),
        )
        .expect("valid geometry");

        assert_eq!(widths, vec![20.0, 20.0]);
    }
}

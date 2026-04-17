use super::super::WorkbenchChromeMetrics;
use super::super::constraints::aggregate_row_constraints;
use super::super::region_state::RegionState;

pub(super) fn compute_window_min_width(
    left: RegionState,
    document: RegionState,
    right: RegionState,
    metrics: &WorkbenchChromeMetrics,
) -> f32 {
    let mut widths = Vec::new();
    if left.visible {
        widths.push(left.constraints);
    }
    widths.push(document.constraints);
    if right.visible {
        widths.push(right.constraints);
    }
    let separators = widths.len().saturating_sub(1) as f32 * metrics.separator_thickness;
    aggregate_row_constraints(&widths).width.resolved().min + separators
}

pub(super) fn compute_window_min_height(
    left: RegionState,
    document: RegionState,
    right: RegionState,
    bottom: RegionState,
    metrics: &WorkbenchChromeMetrics,
) -> f32 {
    let mut min_height = metrics.top_bar_height
        + metrics.separator_thickness
        + metrics.host_bar_height
        + metrics.separator_thickness
        + metrics.status_bar_height
        + metrics.separator_thickness;
    let row_height_constraint =
        aggregate_row_constraints(&[left.constraints, document.constraints, right.constraints]);
    let center_min = row_height_constraint.height.resolved().min;
    if bottom.visible {
        min_height +=
            center_min + bottom.constraints.height.resolved().min + metrics.separator_thickness;
    } else {
        min_height += center_min;
    }
    min_height
}

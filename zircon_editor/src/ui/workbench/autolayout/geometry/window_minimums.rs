use super::super::constraints::aggregate_row_constraints;
use super::super::region_state::RegionState;
use super::super::{
    WorkbenchChromeMetrics, window_min_height_limit_for_height,
    window_min_width_limit_for_logical_width,
};

pub(super) fn compute_window_min_width(
    left: RegionState,
    document: RegionState,
    right: RegionState,
    metrics: &WorkbenchChromeMetrics,
    shell_logical_width: f32,
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
    let content_min_width = aggregate_row_constraints(&widths).width.resolved().min + separators;
    content_min_width.min(window_min_width_limit_for_logical_width(
        shell_logical_width,
    ))
}

pub(super) fn compute_window_min_height(
    left: RegionState,
    document: RegionState,
    right: RegionState,
    bottom: RegionState,
    metrics: &WorkbenchChromeMetrics,
    shell_height: f32,
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
    min_height.min(window_min_height_limit_for_height(shell_height))
}

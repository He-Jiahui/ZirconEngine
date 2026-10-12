use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host) struct HierarchyRowMetrics {
    pub row_x: f32,
    pub row_y: f32,
    pub row_height: f32,
    pub row_gap: f32,
    pub row_width_inset: f32,
}

impl Default for HierarchyRowMetrics {
    fn default() -> Self {
        current_hierarchy_row_metrics()
    }
}

pub(in crate::ui::retained_host) fn current_hierarchy_row_metrics() -> HierarchyRowMetrics {
    hierarchy_row_metrics_from_host_metrics(current_host_metrics())
}

pub(in crate::ui::retained_host) fn hierarchy_row_metrics_from_host_metrics(
    metrics: HostControlMetrics,
) -> HierarchyRowMetrics {
    let horizontal_inset = metrics.gap_m.max(0.0);
    let row_height = (metrics.row_height - metrics.border_width * 2.0)
        .max(metrics.line_height(metrics.font_body))
        .max(0.0);
    HierarchyRowMetrics {
        row_x: horizontal_inset,
        row_y: horizontal_inset,
        row_height,
        row_gap: metrics.border_width.max(0.0),
        row_width_inset: horizontal_inset * 2.0,
    }
}

pub(in crate::ui::retained_host) fn hierarchy_row_y(
    metrics: HierarchyRowMetrics,
    index: usize,
    scroll_px: f32,
) -> f32 {
    metrics.row_y + index as f32 * (metrics.row_height + metrics.row_gap) - scroll_px.max(0.0)
}

pub(in crate::ui::retained_host) fn hierarchy_row_width(
    pane_width: f32,
    metrics: HierarchyRowMetrics,
) -> f32 {
    (pane_width - metrics.row_width_inset).max(0.0)
}

pub(in crate::ui::retained_host) fn hierarchy_content_height(
    item_count: usize,
    metrics: HierarchyRowMetrics,
) -> f32 {
    if item_count == 0 {
        0.0
    } else {
        metrics.row_y * 2.0
            + item_count as f32 * metrics.row_height
            + (item_count as f32 - 1.0) * metrics.row_gap
    }
}

#[cfg(test)]
#[path = "tests/row_metrics.rs"]
mod tests;

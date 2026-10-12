use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

pub(super) const MUI_X_TREE_ROW_COUNT: i32 = 3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct TreeViewRowMetrics {
    pub horizontal_inset: f32,
    pub indent_step: f32,
    pub row_gap: f32,
    pub row_radius: f32,
    pub marker_inset: f32,
    pub marker_min_edge: f32,
    pub marker_max_edge: f32,
}

pub(super) fn tree_view_row_metrics() -> TreeViewRowMetrics {
    tree_view_row_metrics_from_host(current_host_metrics())
}

pub(crate) fn tree_view_header_height(rect: &FrameRect, node: &TemplatePaneNodeData) -> f32 {
    if node.text.trim().is_empty() {
        return 0.0;
    }
    let metrics = current_host_metrics();
    (rect.height * 0.2)
        .max(metrics.line_height(metrics.font_body) + metrics.gap_s)
        .min(rect.height.max(0.0))
}

fn tree_view_row_metrics_from_host(metrics: HostControlMetrics) -> TreeViewRowMetrics {
    let border_width = metrics.border_width.max(0.0);
    let horizontal_inset = metrics.gap_s.max(0.0);
    let marker_inset = (metrics.gap_s - border_width).max(0.0);
    TreeViewRowMetrics {
        horizontal_inset,
        indent_step: (metrics.gap_m - border_width * 2.0).max(border_width),
        row_gap: border_width,
        row_radius: metrics.radius_control.max(0.0),
        marker_inset,
        marker_min_edge: marker_inset,
        marker_max_edge: (metrics.gap_m - border_width * 2.0).max(marker_inset),
    }
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;

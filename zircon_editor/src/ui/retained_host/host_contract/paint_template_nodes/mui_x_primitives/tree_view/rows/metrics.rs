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
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::METRICS;

    #[test]
    fn tree_view_metrics_follow_the_workbench_density_baseline() {
        let metrics = tree_view_row_metrics_from_host(METRICS);

        assert_eq!(metrics.horizontal_inset, 4.0);
        assert_eq!(metrics.indent_step, 6.0);
        assert_eq!(metrics.row_gap, 1.0);
        assert_eq!(metrics.row_radius, 4.0);
        assert_eq!(metrics.marker_inset, 3.0);
        assert_eq!(metrics.marker_min_edge, 3.0);
        assert_eq!(metrics.marker_max_edge, 6.0);
    }

    #[test]
    fn tree_view_metrics_reflow_from_compact_host_density() {
        let mut host = METRICS;
        host.gap_s = 3.0;
        host.gap_m = 6.0;
        host.border_width = 1.5;
        host.radius_control = 3.0;
        let metrics = tree_view_row_metrics_from_host(host);

        assert_eq!(metrics.horizontal_inset, 3.0);
        assert_eq!(metrics.indent_step, 3.0);
        assert_eq!(metrics.row_gap, 1.5);
        assert_eq!(metrics.row_radius, 3.0);
        assert_eq!(metrics.marker_inset, 1.5);
        assert_eq!(metrics.marker_min_edge, 1.5);
        assert_eq!(metrics.marker_max_edge, 3.0);
    }
}

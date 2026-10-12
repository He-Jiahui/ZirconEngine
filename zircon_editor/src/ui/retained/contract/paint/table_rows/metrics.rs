//! 表格表面圆角和行分隔厚度跟随动态宿主密度；粗边框时圆角可退化为零，不能产生负圆角。

use super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WorkbenchTableRowSurfaceMetrics {
    pub radius: f32,
    pub separator_height: f32,
}

pub(super) fn table_row_surface_metrics() -> WorkbenchTableRowSurfaceMetrics {
    table_row_surface_metrics_from_host(current_host_metrics())
}

fn table_row_surface_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchTableRowSurfaceMetrics {
    WorkbenchTableRowSurfaceMetrics {
        radius: (metrics.radius_control - metrics.border_width).max(0.0),
        separator_height: metrics.border_width,
    }
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;

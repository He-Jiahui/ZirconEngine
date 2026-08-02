use super::super::super::paint_theme::{HostControlMetrics, current_host_metrics};

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
mod tests {
    use super::super::super::super::paint_theme::METRICS;
    use super::*;

    #[test]
    fn table_row_surface_metrics_project_from_host_control_metrics() {
        let mut host = METRICS;
        host.radius_control = 6.0;
        host.border_width = 1.5;

        let metrics = table_row_surface_metrics_from_host(host);

        assert_eq!(metrics.radius, 4.5);
        assert_eq!(metrics.separator_height, 1.5);
    }

    #[test]
    fn table_row_surface_radius_clamps_when_border_exceeds_radius() {
        let mut host = METRICS;
        host.radius_control = 1.0;
        host.border_width = 2.0;

        let metrics = table_row_surface_metrics_from_host(host);

        assert_eq!(metrics.radius, 0.0);
        assert_eq!(metrics.separator_height, 2.0);
    }
}

use super::super::super::super::paint_theme::{HostControlMetrics, current_host_metrics};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WorkbenchTableActionMetrics {
    pub action_column_width: f32,
    pub button_size: f32,
    pub icon_size: f32,
    pub border_width: f32,
    pub radius: f32,
}

pub(super) fn table_action_metrics() -> WorkbenchTableActionMetrics {
    table_action_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes::template_table_rows) fn table_action_column_width()
-> f32 {
    table_action_metrics().action_column_width
}

fn table_action_metrics_from_host(metrics: HostControlMetrics) -> WorkbenchTableActionMetrics {
    let icon_size = metrics.gap_m * 2.0;
    let button_size = icon_size + metrics.gap_s;
    WorkbenchTableActionMetrics {
        action_column_width: button_size + metrics.gap_s,
        button_size,
        icon_size,
        border_width: metrics.border_width,
        radius: metrics.radius_control,
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::super::super::paint_theme::METRICS;
    use super::*;

    #[test]
    fn table_action_metrics_project_from_host_control_metrics() {
        let mut host = METRICS;
        host.gap_s = 5.0;
        host.gap_m = 9.0;
        host.border_width = 1.5;
        host.radius_control = 6.0;

        let metrics = table_action_metrics_from_host(host);

        assert_eq!(metrics.icon_size, 18.0);
        assert_eq!(metrics.button_size, 23.0);
        assert_eq!(metrics.action_column_width, 28.0);
        assert_eq!(metrics.border_width, 1.5);
        assert_eq!(metrics.radius, 6.0);
    }
}

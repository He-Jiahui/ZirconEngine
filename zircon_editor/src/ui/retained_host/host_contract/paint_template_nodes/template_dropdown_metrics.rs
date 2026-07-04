use super::super::paint_theme::{current_host_metrics, HostControlMetrics};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchDropdownMetrics
{
    pub border_width: f32,
    pub radius: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub text_inset_x: f32,
    pub chevron_size: f32,
    pub chevron_right: f32,
    pub chevron_reserve: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_dropdown_metrics(
) -> WorkbenchDropdownMetrics {
    workbench_dropdown_metrics_from_host(current_host_metrics())
}

fn workbench_dropdown_metrics_from_host(metrics: HostControlMetrics) -> WorkbenchDropdownMetrics {
    let chevron_size = (metrics.button_chevron_reserve - metrics.gap_s
        + metrics.border_width * 2.0)
        .max(metrics.font_body);
    let chevron_right = metrics.button_icon_gap;
    WorkbenchDropdownMetrics {
        border_width: metrics.border_width,
        radius: metrics.radius_control,
        font_size: metrics.font_body,
        line_height: metrics.line_height(metrics.font_body),
        text_inset_x: metrics.input_pad[0],
        chevron_size,
        chevron_right,
        chevron_reserve: chevron_size + chevron_right + metrics.gap_s,
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::paint_theme::METRICS;
    use super::*;

    #[test]
    fn dropdown_metrics_project_from_host_control_metrics() {
        let mut host = METRICS;
        host.border_width = 2.0;
        host.radius_control = 6.0;
        host.font_body = 11.0;
        host.line_height_ratio = 1.4;
        host.input_pad = [9.0, 8.0, 5.0, 4.0];
        host.button_chevron_reserve = 24.0;
        host.button_icon_gap = 6.0;
        host.gap_s = 5.0;

        let metrics = workbench_dropdown_metrics_from_host(host);

        assert_eq!(metrics.border_width, 2.0);
        assert_eq!(metrics.radius, 6.0);
        assert_eq!(metrics.font_size, 11.0);
        assert!((metrics.line_height - 15.4).abs() < f32::EPSILON);
        assert_eq!(metrics.text_inset_x, 9.0);
        assert_eq!(metrics.chevron_size, 23.0);
        assert_eq!(metrics.chevron_right, 6.0);
        assert_eq!(metrics.chevron_reserve, 34.0);
    }
}

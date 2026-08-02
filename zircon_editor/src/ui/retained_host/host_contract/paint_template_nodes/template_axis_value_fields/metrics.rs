use super::super::super::paint_theme::{HostControlMetrics, current_host_metrics};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct AxisValueFieldMetrics {
    pub max_height: f32,
    pub radius: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub text_inset_x: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_value_field_metrics()
-> AxisValueFieldMetrics {
    axis_value_field_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_value_field_metrics_from_host(
    metrics: HostControlMetrics,
) -> AxisValueFieldMetrics {
    AxisValueFieldMetrics {
        max_height: (metrics.row_height + metrics.border_width * 2.0).max(0.0),
        radius: metrics.radius_control,
        font_size: metrics.font_body,
        line_height: metrics.line_height(metrics.font_body),
        text_inset_x: metrics.input_pad[0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_value_field_metrics_project_from_host_control_metrics() {
        let mut host = current_host_metrics();
        host.row_height = 28.0;
        host.border_width = 1.5;
        host.radius_control = 3.0;
        host.font_body = 11.0;
        host.line_height_ratio = 1.25;
        host.input_pad[0] = 6.0;

        let metrics = axis_value_field_metrics_from_host(host);

        assert_eq!(metrics.max_height, 31.0);
        assert_eq!(metrics.radius, 3.0);
        assert_eq!(metrics.font_size, 11.0);
        assert_eq!(metrics.line_height, 13.75);
        assert_eq!(metrics.text_inset_x, 6.0);
    }
}

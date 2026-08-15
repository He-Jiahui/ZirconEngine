use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchStatusMetrics
{
    pub font_size: f32,
    pub line_height: f32,
    pub radius: f32,
    pub border_width: f32,
    pub text_inset: f32,
    pub text_value_gap: f32,
    pub icon_glyph_size: f32,
    pub signal_icon_left: f32,
    pub signal_text_gap: f32,
    pub signal_marker_size: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_status_metrics(
) -> WorkbenchStatusMetrics {
    workbench_status_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_status_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchStatusMetrics {
    let font_size = metrics.font_body;
    WorkbenchStatusMetrics {
        font_size,
        line_height: metrics.line_height(font_size),
        radius: metrics.radius_control,
        border_width: metrics.border_width,
        text_inset: metrics.gap_s,
        text_value_gap: metrics.gap_s,
        // Status icons share the compact Icon16 slot with panel buttons.
        icon_glyph_size: (metrics.row_height - metrics.gap_l)
            .min(metrics.row_height)
            .max(1.0),
        signal_icon_left: metrics.gap_m,
        signal_text_gap: metrics.gap_m,
        signal_marker_size: metrics.gap_m,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_font_size() -> f32 {
    workbench_status_metrics().font_size
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_chip_radius() -> f32
{
    workbench_status_metrics().radius
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_icon_button_radius(
) -> f32 {
    workbench_status_metrics().radius
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_icon_glyph_size(
) -> f32 {
    workbench_status_metrics().icon_glyph_size
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_line_height() -> f32
{
    workbench_status_metrics().line_height
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::METRICS;

    #[test]
    fn workbench_status_metrics_project_from_host_control_metrics() {
        let mut host = METRICS;
        host.font_body = 11.0;
        host.font_large = 15.0;
        host.line_height_ratio = 1.25;
        host.radius_control = 3.0;
        host.border_width = 1.5;
        host.gap_s = 5.0;
        host.gap_m = 9.0;
        host.gap_l = 13.0;
        host.row_height = 28.0;

        let metrics = workbench_status_metrics_from_host(host);

        assert_eq!(metrics.font_size, 11.0);
        assert_eq!(metrics.line_height, 13.75);
        assert_eq!(metrics.radius, 3.0);
        assert_eq!(metrics.border_width, 1.5);
        assert_eq!(metrics.text_inset, 5.0);
        assert_eq!(metrics.text_value_gap, 5.0);
        assert_eq!(metrics.icon_glyph_size, 15.0);
        assert_eq!(metrics.signal_icon_left, 9.0);
        assert_eq!(metrics.signal_text_gap, 9.0);
        assert_eq!(metrics.signal_marker_size, 9.0);
    }
}

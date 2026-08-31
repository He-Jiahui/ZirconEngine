use super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchPopupRowMetrics
{
    pub font_size: f32,
    pub line_height: f32,
    pub text_left: f32,
    pub text_right: f32,
    pub text_top: f32,
    pub text_bottom: f32,
    pub surface_radius: f32,
    pub outline_width: f32,
    pub adornment_right: f32,
    pub adornment_size: f32,
    pub adornment_reserved_width: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_popup_row_metrics(
) -> WorkbenchPopupRowMetrics {
    workbench_popup_row_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_popup_row_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchPopupRowMetrics {
    let font_size = metrics.font_body;
    let line_height = metrics.line_height(font_size);
    let adornment_size = metrics.font_large;
    WorkbenchPopupRowMetrics {
        font_size,
        line_height,
        text_left: metrics.input_pad[0],
        text_right: metrics.input_pad[1],
        text_top: metrics.input_pad[2],
        text_bottom: metrics.input_pad[3],
        surface_radius: (metrics.radius_control - metrics.border_width).max(0.0),
        outline_width: metrics.border_width,
        adornment_right: metrics.gap_l,
        adornment_size,
        adornment_reserved_width: adornment_size + metrics.gap_m * 2.0,
    }
}

use super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchTooltipMetrics
{
    pub bubble_min_width: f32,
    pub bubble_max_width: f32,
    pub bubble_height: f32,
    pub radius: f32,
    pub border_width: f32,
    pub shadow_offset_y: f32,
    pub text_left: f32,
    pub title_top: f32,
    pub body_top: f32,
    pub title_font_size: f32,
    pub title_line_height: f32,
    pub body_font_size: f32,
    pub body_line_height: f32,
    pub arrow_size: f32,
    pub arrow_min: f32,
    pub arrow_max: f32,
    pub icon_size: f32,
    pub icon_min: f32,
    pub icon_max: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tooltip_metrics(
) -> WorkbenchTooltipMetrics {
    tooltip_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tooltip_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchTooltipMetrics {
    let title_font_size = metrics.font_body + metrics.border_width * 2.0;
    let body_font_size = metrics.font_body + metrics.border_width;
    WorkbenchTooltipMetrics {
        bubble_min_width: metrics.row_height * 3.0,
        bubble_max_width: metrics.row_height * 10.0,
        bubble_height: metrics.row_height + metrics.gap_l + metrics.gap_m + metrics.border_width,
        radius: metrics.radius_control,
        border_width: metrics.border_width,
        shadow_offset_y: metrics.gap_m,
        text_left: metrics.gap_m,
        title_top: (metrics.gap_m - metrics.border_width).max(0.0),
        body_top: (metrics.row_height - metrics.border_width).max(0.0),
        title_font_size,
        title_line_height: title_font_size + metrics.border_width * 2.0,
        body_font_size,
        body_line_height: body_font_size + metrics.border_width * 2.0,
        arrow_size: metrics.gap_m,
        arrow_min: metrics.gap_s,
        arrow_max: metrics.gap_l + metrics.border_width * 2.0,
        icon_size: (metrics.row_height - metrics.gap_s - metrics.border_width * 2.0)
            .max(metrics.font_body),
        icon_min: metrics.font_body,
        icon_max: metrics.row_height,
    }
}

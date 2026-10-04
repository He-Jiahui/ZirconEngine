//! 拖拽提示内容的宿主密度快照；图标、文本与落点线使用统一主题尺寸，避免引入第二套 DPI 缩放。

use super::super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct DragOverlayMetrics {
    pub border_width: f32,
    pub preview_radius: f32,
    pub icon_radius: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub icon_left: f32,
    pub icon_size: f32,
    pub text_left_with_icon: f32,
    pub text_right_inset: f32,
    pub indicator_thickness: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn drag_overlay_metrics(
) -> DragOverlayMetrics {
    drag_overlay_metrics_from_host(current_host_metrics())
}

fn drag_overlay_metrics_from_host(metrics: HostControlMetrics) -> DragOverlayMetrics {
    let icon_size = (metrics.row_height - metrics.gap_l).max(0.0);
    DragOverlayMetrics {
        border_width: metrics.border_width,
        preview_radius: metrics.radius_control,
        icon_radius: metrics.radius_control.min(icon_size * 0.5),
        font_size: metrics.font_body,
        line_height: metrics.line_height(metrics.font_body),
        icon_left: metrics.button_pad_x,
        icon_size,
        text_left_with_icon: metrics.button_pad_x + icon_size + metrics.button_icon_gap,
        text_right_inset: metrics.button_pad_x,
        indicator_thickness: metrics.tab_underline_height,
    }
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;

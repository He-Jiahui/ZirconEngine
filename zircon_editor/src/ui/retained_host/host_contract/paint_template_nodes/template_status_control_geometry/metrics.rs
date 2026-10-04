//! 把当前帧密度统一投影给 chip、图标按钮和状态信号，使绘制尺寸与点击布局使用同一套 token。

use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

/// 状态栏子控件共用的帧内几何快照；绘制前从宿主主题获取以跟随密度变更。
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
#[path = "tests/metrics.rs"]
mod tests;

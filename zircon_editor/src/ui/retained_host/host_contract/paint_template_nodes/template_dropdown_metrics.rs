//! 一次下拉绘制所用的宿主密度快照；表面、文字和箭头共享该值以避免同帧尺度漂移。

use super::super::paint_theme::{current_host_metrics, HostControlMetrics};

#[derive(Clone, Copy, Debug, PartialEq)]
/// 下拉框表面、标签和箭头共享的一次绘制尺度；构造入口应在命令接管时只调用一次。
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
#[path = "tests/template_dropdown_metrics.rs"]
mod tests;

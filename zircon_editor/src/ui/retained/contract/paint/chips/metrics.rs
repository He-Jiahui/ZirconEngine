//! 共享宿主密度生成 chip 文字、边框、圆角、箭头与文字预留；chip glyph 域使用同一组值。

use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

#[derive(Clone, Copy, Debug, PartialEq)]
/// 标签、表面与 SVG 箭头消费的同一密度投影，字段需一起随宿主指标变化。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchChipMetrics {
    pub font_size: f32,
    pub line_height: f32,
    pub radius: f32,
    pub border_width: f32,
    pub text_left: f32,
    pub text_right: f32,
    pub chevron_size: f32,
    pub chevron_right: f32,
    pub chevron_reserve: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_chip_metrics(
) -> WorkbenchChipMetrics {
    workbench_chip_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_chip_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchChipMetrics {
    let font_size = metrics.font_body;
    let chevron_size = metrics.font_body + metrics.border_width * 2.0;
    let chevron_right = metrics.gap_m;
    WorkbenchChipMetrics {
        font_size,
        line_height: metrics.line_height(font_size),
        radius: metrics.radius_control,
        border_width: metrics.border_width,
        text_left: metrics.gap_m + metrics.border_width * 2.0,
        text_right: metrics.gap_m,
        chevron_size,
        chevron_right,
        chevron_reserve: chevron_size + chevron_right + metrics.gap_s,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_font_size() -> f32 {
    workbench_chip_metrics().font_size
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_line_height() -> f32 {
    workbench_chip_metrics().line_height
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_radius() -> f32 {
    workbench_chip_metrics().radius
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_border_width() -> f32
{
    workbench_chip_metrics().border_width
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_text_left() -> f32 {
    workbench_chip_metrics().text_left
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_text_right() -> f32 {
    workbench_chip_metrics().text_right
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_chevron_size() -> f32
{
    workbench_chip_metrics().chevron_size
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_chevron_right() -> f32
{
    workbench_chip_metrics().chevron_right
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_chevron_reserve(
) -> f32 {
    workbench_chip_metrics().chevron_reserve
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;

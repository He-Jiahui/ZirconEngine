use super::super::super::super::super::super::{
    data::{FrameRect, TemplatePaneNodeData},
    paint_text::measure_runtime_text_width,
};
use super::super::align::pixel_aligned;
use super::super::metrics::{divider_font_size, divider_wrapped_label_width};
use super::align::{divider_text_align, divider_text_align_ratio};

/// 用运行时实际字形宽度确定水平线段的标签缺口；两个线段随后在该边界两侧绘制。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn horizontal_label_bounds(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    line_start: f32,
    line_end: f32,
    label: &str,
) -> (f32, f32) {
    let label_width = measured_horizontal_label_width(node, rect, label, line_end - line_start);
    let label_left = horizontal_label_left(node, line_start, line_end, label_width);
    (label_left, (label_left + label_width).min(line_end))
}

fn horizontal_label_left(
    node: &TemplatePaneNodeData,
    line_start: f32,
    line_end: f32,
    label_width: f32,
) -> f32 {
    let available = (line_end - line_start).max(0.0);
    let remaining = (available - label_width).max(0.0);
    let ratio = divider_text_align_ratio(divider_text_align(node));
    pixel_aligned(line_start + remaining * ratio)
}

fn measured_horizontal_label_width(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    label: &str,
    available_width: f32,
) -> f32 {
    let font_size = divider_font_size(node, rect.height);
    let text_width = measure_runtime_text_width(label, font_size);
    divider_wrapped_label_width(text_width, available_width)
}

#[cfg(test)]
#[path = "tests/horizontal.rs"]
mod tests;

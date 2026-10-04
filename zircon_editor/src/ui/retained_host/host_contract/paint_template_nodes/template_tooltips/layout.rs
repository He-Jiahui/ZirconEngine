//! 气泡宽度由 runtime 文字度量决定，并受作者容器与主题宽度上限约束；绘制保留已有分数 DPI 坐标。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::tooltip_metrics;
use super::text::{tooltip_body, tooltip_title};
use crate::ui::retained_host::host_contract::paint_text::measure_runtime_text_width;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn has_paintable_tooltip_extent(
    rect: &FrameRect,
) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn frame_is_within(
    outer: &FrameRect,
    inner: &FrameRect,
) -> bool {
    has_paintable_tooltip_extent(outer)
        && has_paintable_tooltip_extent(inner)
        && inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

/// 使用与文本绘制相同的修剪后内容计算气泡；有正文和只有标题分别消费不同高度。
/// layout_offset 只移动气泡，调用方还要检查结果仍被完整 tooltip 容器包含。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tooltip_bubble_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let metrics = tooltip_metrics();
    let width = tooltip_bubble_width(node, rect, metrics);
    FrameRect {
        x: rect.x + (rect.width - width).max(0.0) * 0.5 + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width,
        height: tooltip_bubble_height(node, metrics).min(rect.height.max(0.0)),
    }
}

fn tooltip_bubble_height(
    node: &TemplatePaneNodeData,
    metrics: super::metrics::WorkbenchTooltipMetrics,
) -> f32 {
    if tooltip_body(node).is_empty() {
        // Icon-button labels are title-only; retain only the space required to paint that line.
        metrics.title_top + metrics.title_line_height + metrics.border_width * 2.0
    } else {
        metrics.bubble_height
    }
}

fn tooltip_bubble_width(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    metrics: super::metrics::WorkbenchTooltipMetrics,
) -> f32 {
    let title_width = measure_runtime_text_width(tooltip_title(node), metrics.title_font_size);
    let body_width = measure_runtime_text_width(tooltip_body(node), metrics.body_font_size);
    let desired_width = title_width.max(body_width) + metrics.text_left * 2.0;
    let available_width = rect.width.max(0.0);
    let maximum_width = metrics
        .bubble_max_width
        .max(metrics.bubble_min_width)
        .min(available_width);
    let minimum_width = metrics.bubble_min_width.min(maximum_width);

    // Content leads the bubble width, while authored bounds remain authoritative.
    desired_width.clamp(minimum_width, maximum_width)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paint_rect(
    rect: &FrameRect,
) -> FrameRect {
    rect.clone()
}

#[cfg(test)]
#[path = "tests/layout_fractional_geometry_tests.rs"]
mod fractional_geometry_tests;

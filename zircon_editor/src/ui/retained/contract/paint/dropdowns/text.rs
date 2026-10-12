//! 下拉标签取共享节点文字；空值时可用首个选项作占位，并为尾部箭头保留稳定文字空间。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchDropdownStyle;
use super::super::template_dropdown_metrics::WorkbenchDropdownMetrics;
use super::super::template_node_labels::template_node_label;
use super::geometry::{frame_is_within, has_paintable_dropdown_extent};
use crate::ui::retained_host::host_contract::paint_geometry::intersect;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

/// 入口已传入占位样式及同帧指标；文字框必须完整装入下拉本体，并继续受 clip 限制。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_dropdown_label(
    commands: &mut Vec<HostPaintCommand>,
    label: String,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    style: &WorkbenchDropdownStyle,
    metrics: &WorkbenchDropdownMetrics,
) {
    if label.trim().is_empty() {
        return;
    }
    let text_rect = FrameRect {
        x: rect.x + metrics.text_inset_x,
        y: rect.y + (rect.height - metrics.line_height).max(0.0) * 0.5,
        width: (rect.width - metrics.text_inset_x - metrics.chevron_reserve).max(0.0),
        height: metrics.line_height.max(0.0),
    };
    if !has_paintable_dropdown_extent(&text_rect)
        || !frame_is_within(rect, &text_rect)
        || intersect(&text_rect, clip).is_none()
    {
        return;
    }
    commands.push(HostPaintCommand::text(
        text_rect,
        Some(clip.clone()),
        order,
        label,
        style.text,
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

/// 共享标签优先；无文字时使用第一个选项作为占位，而非已选择索引的显示值。
pub(super) fn dropdown_label(node: &TemplatePaneNodeData) -> (String, bool) {
    let label = template_node_label(node, None);
    if !label.trim().is_empty() {
        return (label, false);
    }
    let fallback = node
        .options
        .get(0)
        .map(|value| value.to_string())
        .unwrap_or_default();
    (fallback, true)
}

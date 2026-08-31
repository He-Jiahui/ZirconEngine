use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchDropdownStyle;
use super::super::template_dropdown_metrics::WorkbenchDropdownMetrics;
use super::super::template_node_labels::template_node_label;
use super::geometry::{frame_is_within, has_paintable_dropdown_extent};
use crate::ui::retained_host::host_contract::paint_geometry::intersect;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

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

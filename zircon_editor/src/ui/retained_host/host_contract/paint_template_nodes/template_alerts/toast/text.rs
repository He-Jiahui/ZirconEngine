use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_node_labels::template_node_label;
use super::super::layout::{WorkbenchToastMetrics, frame_is_within, toast_text_rect};
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) fn push_toast_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    icon: Option<&FrameRect>,
    close: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    has_action: bool,
    metrics: WorkbenchToastMetrics,
    opacity: f32,
) {
    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }

    let Some(text_rect) = toast_text_rect(rect, icon, close, has_action, metrics) else {
        return;
    };
    if !frame_is_within(&text_rect, rect) || text_rect.height < metrics.line_height {
        return;
    }

    commands.push(HostPaintCommand::text(
        text_rect,
        Some(clip.clone()),
        order,
        label,
        color,
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

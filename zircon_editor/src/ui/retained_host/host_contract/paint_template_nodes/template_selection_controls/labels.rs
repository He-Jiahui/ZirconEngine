use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_node_labels::template_node_label;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const SELECTION_FONT_SIZE: f32 = 10.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_selection_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let label = template_node_label(node, None);
    if label.trim().is_empty() || rect.width <= 0.5 || rect.height <= 0.5 {
        return;
    }
    commands.push(HostPaintCommand::text(
        rect,
        Some(clip.clone()),
        order,
        label,
        color,
        SELECTION_FONT_SIZE,
        SELECTION_FONT_SIZE * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

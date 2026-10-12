use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_buttons::push_button_commands;
use super::super::super::template_material_feedback::push_material_feedback_primitive_commands;
use super::super::super::template_segmented_controls::push_segmented_control_commands;
use super::super::super::template_selection_controls::push_selection_control_commands;
use super::super::super::template_shell_panels::push_shell_panel_commands;

pub(super) fn push_primary_specialized_template_node_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    node_clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    push_material_feedback_primitive_commands(commands, node, rect, node_clip, order, opacity)
        || push_shell_panel_commands(commands, node, rect, node_clip, order, opacity)
        || push_selection_control_commands(commands, node, rect, node_clip, order, opacity)
        || push_segmented_control_commands(commands, node, rect, node_clip, order, opacity)
        || push_button_commands(commands, node, rect, node_clip, order, opacity)
}

mod backdrop;
mod circular_progress;
mod progress;
mod state;

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use backdrop::{is_material_backdrop_node, push_material_backdrop_commands};
use progress::push_material_progress_commands;
use state::is_material_progress_node;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_material_feedback_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if is_material_backdrop_node(node) {
        push_material_backdrop_commands(commands, node, rect, clip, order, opacity);
        return true;
    }
    if is_material_progress_node(node) {
        push_material_progress_commands(commands, node, rect, clip, order, opacity);
        return true;
    }
    false
}

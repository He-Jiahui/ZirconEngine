mod backdrop;
mod circular_progress;
mod metrics;
mod palette;
mod progress;
mod state;

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_geometry::intersect;
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
    let is_backdrop = is_material_backdrop_node(node);
    let is_progress = is_material_progress_node(node);
    if !is_backdrop && !is_progress {
        return false;
    }
    let Some(clip) = intersect(rect, clip) else {
        return true;
    };
    if is_backdrop {
        push_material_backdrop_commands(commands, node, rect, &clip, order, opacity);
    } else {
        push_material_progress_commands(commands, node, rect, &clip, order, opacity);
    }
    true
}

#[cfg(test)]
#[path = "template_material_feedback_tests/mod.rs"]
mod tests;

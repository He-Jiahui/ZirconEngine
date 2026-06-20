use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;

mod commands;
mod eligibility;

use commands::push_surface_commands;
use eligibility::draws_surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use eligibility::is_frame_only_node;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_template_surface_fallback_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    surface_already_drawn: bool,
) {
    if surface_already_drawn || !draws_surface(node) {
        return;
    }

    push_surface_commands(commands, node, rect, clip, order, opacity);
}

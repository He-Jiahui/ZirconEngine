use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_structure::push_base_surface;

use super::haze::push_wall_inner_haze;
use super::panels::push_wall_panel_lines;
use super::shadow::push_wall_top_shadow;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_back_wall_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_base_surface(commands, node, rect, clip, order, opacity);
    push_wall_top_shadow(commands, rect, clip, order + 1, opacity);
    push_wall_panel_lines(commands, rect, clip, order + 2, opacity);
    push_wall_inner_haze(commands, rect, clip, order + 7, opacity);
}

use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_structure::push_base_surface;

use super::depth::push_floor_depth_lines;
use super::shadows::{push_floor_bottom_shadow, push_floor_top_shadow};
use super::sheen::push_floor_warm_sheen;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_floor_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_base_surface(commands, node, rect, clip, order, opacity);
    push_floor_top_shadow(commands, rect, clip, order + 1, opacity);
    push_floor_depth_lines(commands, rect, clip, order + 2, opacity);
    push_floor_warm_sheen(commands, rect, clip, order + 8, opacity);
    push_floor_bottom_shadow(commands, rect, clip, order + 9, opacity);
}

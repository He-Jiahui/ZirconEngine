use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::delete::push_chip_delete_icon;
use super::geometry::chip_frame;
use super::identity::{
    chip_has_avatar, chip_has_icon, chip_is_deletable, is_chip_root_node, is_chip_slot_node,
};
use super::leading::{push_chip_avatar, push_chip_icon};
use super::surface::push_chip_surface;
use super::text::push_chip_label;

const MAX_CHIP_COMMANDS: usize = 14;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chip_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if is_chip_slot_node(node) {
        return true;
    }
    if !is_chip_root_node(node) {
        return false;
    }

    let chip_rect = chip_frame(node, rect);
    if chip_rect.width <= 0.0 || chip_rect.height <= 0.0 {
        return true;
    }

    reserve_chip_command_capacity(commands);
    push_chip_surface(commands, node, &chip_rect, clip, order, opacity);
    if chip_has_avatar(node) {
        push_chip_avatar(commands, node, &chip_rect, clip, order + 1, opacity);
    } else if chip_has_icon(node) {
        push_chip_icon(commands, node, &chip_rect, clip, order + 1, opacity);
    }
    push_chip_label(commands, node, &chip_rect, clip, order + 2, opacity);
    if chip_is_deletable(node) {
        push_chip_delete_icon(commands, node, &chip_rect, clip, order + 3, opacity);
    }

    true
}

fn reserve_chip_command_capacity(commands: &mut Vec<HostPaintCommand>) {
    commands.reserve(MAX_CHIP_COMMANDS);
}

#[cfg(test)]
#[path = "commands/reserve_capacity_tests.rs"]
mod reserve_capacity_tests;

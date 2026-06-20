use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::identity::{is_badge_root_node, is_badge_slot_node};
use super::overlay::push_badge_overlay;
use super::root_label::push_badge_root_label;
use super::root_surface::push_badge_root_surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_badge_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if is_badge_slot_node(node) {
        return true;
    }
    if !is_badge_root_node(node) {
        return false;
    }

    push_badge_root_surface(commands, node, rect, clip, order, opacity);
    push_badge_root_label(commands, node, rect, clip, order + 1, opacity);
    push_badge_overlay(commands, node, rect, clip, order + 2, opacity);
    true
}

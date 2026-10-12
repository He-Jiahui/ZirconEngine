use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::identity::{is_badge_root_node, is_badge_slot_node};
use super::overlay::push_badge_overlay;
use super::root_label::push_badge_root_label;
use super::root_surface::push_badge_root_surface;

const BADGE_PRIMITIVE_COMMAND_CAPACITY: usize = 4;

// 分发器凭返回值决定是否继续通用回退；根按表面、标签、覆盖层顺序绘制，子槽由根接管。
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
    if !rect.x.is_finite()
        || !rect.y.is_finite()
        || !rect.width.is_finite()
        || !rect.height.is_finite()
        || rect.width <= 0.0
        || rect.height <= 0.0
    {
        return true;
    }

    commands.reserve(BADGE_PRIMITIVE_COMMAND_CAPACITY);
    push_badge_root_surface(commands, node, rect, clip, order, opacity);
    push_badge_root_label(commands, node, rect, clip, order + 1, opacity);
    push_badge_overlay(commands, node, rect, clip, order + 2, opacity);
    true
}

#[cfg(test)]
#[path = "tests/sequencing.rs"]
mod tests;

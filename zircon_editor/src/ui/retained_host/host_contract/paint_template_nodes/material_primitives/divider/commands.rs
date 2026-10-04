use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::geometry::divider_is_vertical;
use super::horizontal::push_horizontal_divider;
use super::identity::is_divider_node;
use super::vertical::push_vertical_divider;

const DIVIDER_COMMAND_CAPACITY: usize = 3;

/// 已识别 Divider 即接管；即使完全裁剪也返回 true，避免回退表面覆盖专用线段布局。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_divider_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_divider_node(node) {
        return false;
    }
    let Some(clip) = intersect(rect, clip) else {
        return true;
    };

    commands.reserve(DIVIDER_COMMAND_CAPACITY);
    // MUI Divider is border/pseudo-element geometry, not a filled panel.
    // Emit explicit line segments so inset, middle, and label gaps match the web contract.
    if divider_is_vertical(node, rect) {
        push_vertical_divider(commands, node, rect, &clip, order, opacity);
    } else {
        push_horizontal_divider(commands, node, rect, &clip, order, opacity);
    }
    true
}

#[cfg(test)]
#[path = "tests/commands.rs"]
mod tests;

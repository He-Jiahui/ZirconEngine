use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostPaneInteractionStateData, TemplatePaneNodeData,
};

pub(super) fn template_press_targets_node(
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    let frame = &interaction.pressed_template_frame;
    !node.disabled
        && !interaction.pressed_template_control_id.is_empty()
        && node.control_id == interaction.pressed_template_control_id
        && node.action_id == interaction.pressed_template_action_id
        && node.dispatch_kind == interaction.pressed_template_dispatch_kind
        && origin.x + node.frame.x == frame.x
        && origin.y + node.frame.y == frame.y
        && node.frame.width == frame.width
        && node.frame.height == frame.height
}

pub(super) fn apply_template_press_to_node(
    node: &mut TemplatePaneNodeData,
    origin: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) {
    if template_press_targets_node(node, origin, interaction) {
        node.pressed = true;
    }
}

#[cfg(test)]
#[path = "tests/press.rs"]
mod tests;

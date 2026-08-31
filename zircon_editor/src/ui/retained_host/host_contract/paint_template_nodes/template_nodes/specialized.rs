mod dropdown;
mod primary;
mod secondary;

use super::super::super::data::{FrameRect, HostTextInputFocusData, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_specialized_template_node_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    node_clip: &FrameRect,
    origin: &FrameRect,
    pane_clip: &FrameRect,
    text_input_focus: Option<&HostTextInputFocusData>,
    order: i32,
    opacity: f32,
) -> bool {
    if primary::push_primary_specialized_template_node_commands(
        commands, node, rect, node_clip, order, opacity,
    ) || dropdown::push_dropdown_specialized_template_node_commands(
        commands, node, rect, node_clip, origin, pane_clip, order, opacity,
    ) {
        return true;
    }

    secondary::push_secondary_specialized_template_node_commands(
        commands,
        node,
        rect,
        node_clip,
        text_input_focus,
        order,
        opacity,
    )
}

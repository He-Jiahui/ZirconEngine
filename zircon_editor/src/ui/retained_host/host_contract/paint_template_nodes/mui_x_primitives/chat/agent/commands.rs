use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::bubbles::push_agent_bubbles;
use super::streaming::push_agent_streaming_indicator;
use super::surface::push_agent_surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_agent_chat(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_agent_surface(commands, node, rect, clip, order, opacity);
    push_agent_bubbles(commands, rect, clip, order + 1, opacity);
    push_agent_streaming_indicator(commands, node, rect, clip, order + 3, opacity);
}

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::agent::push_agent_chat;
use super::composer::push_chat_composer;
use super::identity::ChatKind;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chat(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    kind: ChatKind,
) {
    match kind {
        ChatKind::AgentChat => push_agent_chat(commands, node, rect, clip, order, opacity),
        ChatKind::Composer => push_chat_composer(commands, node, rect, clip, order, opacity),
    }
}

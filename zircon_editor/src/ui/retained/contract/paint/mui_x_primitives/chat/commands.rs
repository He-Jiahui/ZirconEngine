use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::agent::push_agent_chat;
use super::composer::push_chat_composer;
use super::identity::ChatKind;

/// 聊天角色在 MUI X 分发后由专属外观接管；普通文本仍可在节点回退链中绘制。
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

#[derive(Clone, Copy)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum ChatKind {
    AgentChat,
    Composer,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chat_kind(
    component_role: &str,
    role: &str,
) -> Option<ChatKind> {
    if super::super::matches_any_role(component_role, role, &["mui-x-agent-chat", "AgentChat"]) {
        Some(ChatKind::AgentChat)
    } else if super::super::matches_any_role(
        component_role,
        role,
        &["mui-x-chat-composer", "ChatComposer"],
    ) {
        Some(ChatKind::Composer)
    } else {
        None
    }
}

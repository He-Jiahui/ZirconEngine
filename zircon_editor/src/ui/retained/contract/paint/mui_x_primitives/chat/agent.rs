#[path = "../../chat/agent/bubbles.rs"]
mod bubbles;
#[path = "../../chat/agent/commands.rs"]
mod commands;
#[path = "../../chat/agent/content.rs"]
mod content;
#[path = "../../chat/agent/messages.rs"]
mod messages;
#[path = "../../chat/agent/streaming.rs"]
mod streaming;
#[path = "../../chat/agent/surface.rs"]
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_agent_chat;

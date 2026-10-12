#[path = "chat/agent.rs"]
mod agent;
#[path = "chat/commands.rs"]
mod commands;
#[path = "chat/composer.rs"]
mod composer;
#[path = "chat/identity.rs"]
mod identity;
#[path = "chat/metrics.rs"]
mod metrics;
#[path = "chat/style.rs"]
mod style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_chat;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use identity::{
    chat_kind, ChatKind,
};

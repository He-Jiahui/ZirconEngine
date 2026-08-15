mod agent;
mod commands;
mod composer;
mod identity;
mod metrics;
mod style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_chat;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use identity::{
    chat_kind, ChatKind,
};

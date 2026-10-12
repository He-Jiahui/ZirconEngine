#[path = "text/commands.rs"]
mod commands;
#[path = "text/decorations.rs"]
mod decorations;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_text_paint_commands;

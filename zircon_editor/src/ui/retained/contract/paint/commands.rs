#[path = "commands/brush.rs"]
mod brush;
#[path = "commands/commands.rs"]
mod commands;
#[path = "commands/image.rs"]
mod image;
#[path = "commands/style.rs"]
mod style;
#[path = "commands/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::{
    push_runtime_command, runtime_render_commands_to_host,
};

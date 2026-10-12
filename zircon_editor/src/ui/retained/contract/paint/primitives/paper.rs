#[path = "paper/commands.rs"]
mod commands;
#[path = "paper/geometry.rs"]
mod geometry;
#[path = "paper/identity.rs"]
mod identity;
#[path = "paper/shadow.rs"]
mod shadow;
#[path = "paper/style.rs"]
mod style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_paper_primitive_commands;

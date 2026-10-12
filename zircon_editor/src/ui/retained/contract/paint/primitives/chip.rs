#[path = "chip/commands.rs"]
mod commands;
#[path = "chip/delete.rs"]
mod delete;
#[path = "chip/geometry.rs"]
mod geometry;
#[path = "chip/identity.rs"]
mod identity;
#[path = "chip/leading.rs"]
mod leading;
#[path = "chip/style.rs"]
mod style;
#[path = "chip/surface.rs"]
mod surface;
#[path = "chip/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_chip_primitive_commands;

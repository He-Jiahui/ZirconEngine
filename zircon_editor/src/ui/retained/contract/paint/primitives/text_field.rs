#[path = "text_field/commands.rs"]
mod commands;
#[path = "text_field/geometry.rs"]
mod geometry;
#[path = "text_field/identity.rs"]
mod identity;
#[path = "text_field/style.rs"]
mod style;
#[path = "text_field/variant.rs"]
mod variant;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_text_field_surface_commands;

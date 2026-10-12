#[path = "../../text_field/commands/entry.rs"]
mod entry;
#[path = "../../text_field/commands/surface.rs"]
mod surface;
#[path = "../../text_field/commands/underline.rs"]
mod underline;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_text_field_surface_commands;

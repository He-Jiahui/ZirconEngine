#[path = "progress/entry.rs"]
mod entry;
#[path = "progress/linear.rs"]
mod linear;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_material_progress_commands;

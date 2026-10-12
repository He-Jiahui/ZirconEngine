#[path = "resource/count.rs"]
mod count;
#[path = "resource/entry.rs"]
mod entry;
#[path = "resource/field.rs"]
mod field;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_resource_row;

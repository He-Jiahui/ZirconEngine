#[path = "grate/entry.rs"]
mod entry;
#[path = "grate/palette.rs"]
mod palette;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_floor_grate_slots;

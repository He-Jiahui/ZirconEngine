#[path = "circular_progress/cache.rs"]
mod cache;
#[path = "circular_progress/entry.rs"]
mod entry;
#[path = "../../../../contract/paint/feedback/circular_progress/geometry.rs"]
mod geometry;
#[path = "circular_progress/key.rs"]
mod key;
#[path = "../../../../contract/paint/feedback/circular_progress/pixels.rs"]
mod pixels;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_circular_progress_command;

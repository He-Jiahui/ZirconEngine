#[path = "bindings.rs"]
mod bindings;
#[path = "clip_range.rs"]
mod clip_range;
#[path = "input.rs"]
mod input;
#[path = "spatial.rs"]
mod spatial;
#[path = "tracks.rs"]
mod tracks;
#[path = "values.rs"]
mod values;

pub(crate) use tracks::{validate_source_descriptor, validate_source_descriptor_for_graph};

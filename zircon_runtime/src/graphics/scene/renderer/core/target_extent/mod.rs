#[path = "texture_extent.rs"]
mod texture_extent;
#[path = "viewport_size.rs"]
mod viewport_size;

pub(super) use texture_extent::texture_extent;
pub(super) use viewport_size::viewport_size;

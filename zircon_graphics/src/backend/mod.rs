//! GPU device and surface management.

mod render_backend;

pub use render_backend::RuntimePreviewRenderer;
pub(crate) use render_backend::{read_texture_rgba, OffscreenTarget, RenderBackend};

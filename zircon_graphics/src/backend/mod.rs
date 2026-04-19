//! GPU device and surface management.

mod render_backend;

pub(crate) use render_backend::{
    read_buffer_u32s, read_texture_rgba, OffscreenTarget, RenderBackend,
};

//! GPU device and surface management.

mod render_backend;

#[cfg(test)]
pub(crate) use render_backend::RenderBackendConfig;
pub(crate) use render_backend::{
    read_texture_rgba, GraphicsDebuggerCaptureStop, OffscreenTarget, RenderBackend, ViewportSurface,
};

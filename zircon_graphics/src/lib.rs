//! Core rendering, viewport control, and host-agnostic GPU services.

mod render_backend;
mod scene_renderer;
mod service;
mod types;
mod viewport;

pub use render_backend::RuntimePreviewRenderer;
pub use scene_renderer::SceneRenderer;
pub use service::RenderService;
pub use types::{
    EditorOrRuntimeFrame, GizmoAxis, GpuResourceHandle, GraphicsError, ViewportFeedback,
    ViewportFrame, ViewportInput, ViewportState,
};
pub use viewport::ViewportController;

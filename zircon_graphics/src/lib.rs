//! Core rendering, viewport control, and host-agnostic GPU services.

mod backend;
mod host;
mod scene;
mod service;
mod types;
mod viewport;

pub use backend::RuntimePreviewRenderer;
pub use host::{
    create_render_service, create_runtime_preview_renderer, create_shared_texture_render_service,
    module_descriptor, WgpuDriver, WgpuRenderingManager, GRAPHICS_MODULE_NAME,
    RENDERING_MANAGER_NAME, WGPU_DRIVER_NAME,
};
pub use scene::SceneRenderer;
pub use service::{RenderService, SharedTextureRenderService};
pub use types::{
    EditorOrRuntimeFrame, GizmoAxis, GpuResourceHandle, GraphicsError, ViewportFeedback,
    ViewportFrame, ViewportFrameTextureHandle, ViewportInput, ViewportState,
};
pub use viewport::ViewportController;

#[cfg(test)]
mod tests;

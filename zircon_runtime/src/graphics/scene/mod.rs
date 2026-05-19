//! Scene rasterization and resource streaming.

#[cfg(test)]
mod render_product_streamer_tests;
mod resources;
#[path = "scene_renderer/mod.rs"]
mod scene_renderer;

pub use scene_renderer::SceneRenderer;
#[cfg(test)]
pub(crate) use scene_renderer::ViewportOverlayRenderer;
pub(crate) use scene_renderer::{
    anti_alias, cluster_buffer_bytes_for_size, cluster_dimensions_for_size, create_depth_texture,
    GBUFFER_ALBEDO_FORMAT, NORMAL_FORMAT, OFFSCREEN_FORMAT,
};
pub use scene_renderer::{
    RenderGraphExecutionResources, RenderPassExecutionContext, RenderPassExecutor,
    RenderPassExecutorFn, RenderPassExecutorId, RenderPassExecutorRegistration,
    RenderPassGpuExecutionContext,
};

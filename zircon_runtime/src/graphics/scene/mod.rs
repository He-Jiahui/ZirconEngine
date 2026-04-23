//! Scene rasterization and resource streaming.

mod resources;
#[path = "scene_renderer/mod.rs"]
mod scene_renderer;

pub use scene_renderer::SceneRenderer;
#[cfg(test)]
pub(crate) use scene_renderer::ViewportOverlayRenderer;
pub(crate) use scene_renderer::{
    cluster_buffer_bytes_for_size, cluster_dimensions_for_size, create_depth_texture,
    HybridGiScenePrepareResourcesSnapshot, GBUFFER_ALBEDO_FORMAT, NORMAL_FORMAT, OFFSCREEN_FORMAT,
};

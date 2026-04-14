//! Scene rasterization and resource streaming.

mod scene_renderer;

pub use scene_renderer::SceneRenderer;
pub(crate) use scene_renderer::{
    create_depth_texture, ResourceStreamer, SceneRendererCore, OFFSCREEN_FORMAT,
};

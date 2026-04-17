use std::sync::Arc;

use winit::window::Window;
use zircon_asset::ProjectAssetManager;

use crate::scene::{ResourceStreamer, SceneRendererCore};
use crate::types::GraphicsError;

use super::render_backend::RenderBackend;
use super::runtime_preview_renderer::RuntimePreviewRenderer;

impl RuntimePreviewRenderer {
    pub fn new(
        window: Arc<dyn Window>,
        asset_manager: Arc<ProjectAssetManager>,
    ) -> Result<Self, GraphicsError> {
        let (backend, surface_state) = RenderBackend::new_with_surface(window)?;
        let scene_renderer =
            SceneRendererCore::new(&backend.device, &backend.queue, surface_state.config.format);
        let streamer = ResourceStreamer::new(
            asset_manager,
            &backend.device,
            &backend.queue,
            &scene_renderer.texture_bind_group_layout,
        );

        Ok(Self {
            backend,
            surface_state,
            scene_renderer,
            streamer,
        })
    }
}

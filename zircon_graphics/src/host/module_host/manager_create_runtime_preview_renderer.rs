use std::sync::Arc;

use winit::window::Window;
use zircon_asset::ProjectAssetManager;

use crate::{GraphicsError, RuntimePreviewRenderer};

use super::wgpu_rendering_manager::WgpuRenderingManager;

impl WgpuRenderingManager {
    pub fn create_runtime_preview_renderer(
        &self,
        window: Arc<dyn Window>,
        asset_manager: Arc<ProjectAssetManager>,
    ) -> Result<RuntimePreviewRenderer, GraphicsError> {
        RuntimePreviewRenderer::new(window, asset_manager)
    }
}

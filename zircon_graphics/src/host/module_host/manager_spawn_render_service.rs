use std::sync::Arc;

use zircon_asset::ProjectAssetManager;

use crate::{GraphicsError, RenderService};

use super::wgpu_rendering_manager::WgpuRenderingManager;

impl WgpuRenderingManager {
    pub fn spawn_render_service(
        &self,
        asset_manager: Arc<ProjectAssetManager>,
    ) -> Result<RenderService, GraphicsError> {
        RenderService::spawn(asset_manager)
    }
}

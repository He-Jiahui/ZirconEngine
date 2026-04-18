use std::sync::Arc;

use zircon_asset::ProjectAssetManager;

use crate::{GraphicsError, RenderService, ViewportIconSource};

use super::wgpu_rendering_manager::WgpuRenderingManager;

impl WgpuRenderingManager {
    pub fn spawn_render_service_with_icon_source(
        &self,
        asset_manager: Arc<ProjectAssetManager>,
        icon_source: Arc<dyn ViewportIconSource>,
    ) -> Result<RenderService, GraphicsError> {
        RenderService::spawn_with_icon_source(asset_manager, Some(icon_source))
    }
}

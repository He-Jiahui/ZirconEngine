use std::sync::Arc;

use zircon_asset::ProjectAssetManager;

use crate::{GraphicsError, SharedTextureRenderService, ViewportIconSource};

use super::wgpu_rendering_manager::WgpuRenderingManager;

impl WgpuRenderingManager {
    pub fn spawn_shared_texture_render_service_with_icon_source(
        &self,
        device: wgpu::Device,
        queue: wgpu::Queue,
        asset_manager: Arc<ProjectAssetManager>,
        icon_source: Arc<dyn ViewportIconSource>,
    ) -> Result<SharedTextureRenderService, GraphicsError> {
        SharedTextureRenderService::spawn_with_device_and_icon_source(
            device,
            queue,
            asset_manager,
            Some(icon_source),
        )
    }
}

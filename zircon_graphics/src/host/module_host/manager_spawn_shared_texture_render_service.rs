use std::sync::Arc;

use zircon_asset::ProjectAssetManager;

use crate::{GraphicsError, SharedTextureRenderService};

use super::wgpu_rendering_manager::WgpuRenderingManager;

impl WgpuRenderingManager {
    pub fn spawn_shared_texture_render_service(
        &self,
        device: wgpu::Device,
        queue: wgpu::Queue,
        asset_manager: Arc<ProjectAssetManager>,
    ) -> Result<SharedTextureRenderService, GraphicsError> {
        SharedTextureRenderService::spawn_with_device(device, queue, asset_manager)
    }
}

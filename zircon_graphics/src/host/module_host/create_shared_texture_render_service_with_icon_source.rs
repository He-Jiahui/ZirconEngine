use std::sync::Arc;

use zircon_core::CoreHandle;

use crate::{GraphicsError, SharedTextureRenderService, ViewportIconSource};

use super::resolve_project_asset_manager::resolve_project_asset_manager;

pub fn create_shared_texture_render_service_with_icon_source(
    core: &CoreHandle,
    device: wgpu::Device,
    queue: wgpu::Queue,
    icon_source: Arc<dyn ViewportIconSource>,
) -> Result<SharedTextureRenderService, GraphicsError> {
    let asset_manager = resolve_project_asset_manager(core)?;
    SharedTextureRenderService::spawn_with_device_and_icon_source(
        device,
        queue,
        asset_manager,
        Some(icon_source),
    )
}

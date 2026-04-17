use zircon_core::CoreHandle;

use crate::{GraphicsError, SharedTextureRenderService};

use super::resolve_project_asset_manager::resolve_project_asset_manager;

pub fn create_shared_texture_render_service(
    core: &CoreHandle,
    device: wgpu::Device,
    queue: wgpu::Queue,
) -> Result<SharedTextureRenderService, GraphicsError> {
    let asset_manager = resolve_project_asset_manager(core)?;
    SharedTextureRenderService::spawn_with_device(device, queue, asset_manager)
}

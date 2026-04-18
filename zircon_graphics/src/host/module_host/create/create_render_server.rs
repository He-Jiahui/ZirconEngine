use std::sync::Arc;

use zircon_core::CoreHandle;
use zircon_render_server::RenderServer;

use crate::{GraphicsError, WgpuRenderServer};

use super::resolve_project_asset_manager::resolve_project_asset_manager;

pub fn create_render_server(core: &CoreHandle) -> Result<Arc<dyn RenderServer>, GraphicsError> {
    let asset_manager = resolve_project_asset_manager(core)?;
    Ok(Arc::new(WgpuRenderServer::new(asset_manager)?))
}

use std::sync::Arc;

use zircon_core::CoreHandle;
use zircon_framework::render::RenderFramework;

use crate::{GraphicsError, WgpuRenderFramework};

use super::resolve_project_asset_manager::resolve_project_asset_manager;

pub fn create_render_framework(
    core: &CoreHandle,
) -> Result<Arc<dyn RenderFramework>, GraphicsError> {
    let asset_manager = resolve_project_asset_manager(core)?;
    Ok(Arc::new(WgpuRenderFramework::new(asset_manager)?))
}

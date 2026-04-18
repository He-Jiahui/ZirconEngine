use std::sync::Arc;

use zircon_core::CoreHandle;

use crate::{GraphicsError, RenderService, ViewportIconSource};

use super::resolve_project_asset_manager::resolve_project_asset_manager;

pub fn create_render_service_with_icon_source(
    core: &CoreHandle,
    icon_source: Arc<dyn ViewportIconSource>,
) -> Result<RenderService, GraphicsError> {
    let asset_manager = resolve_project_asset_manager(core)?;
    RenderService::spawn_with_icon_source(asset_manager, Some(icon_source))
}

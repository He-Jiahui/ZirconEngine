use crate::{GraphicsError, RenderService};
use zircon_core::CoreHandle;

use super::resolve_project_asset_manager::resolve_project_asset_manager;

pub fn create_render_service(core: &CoreHandle) -> Result<RenderService, GraphicsError> {
    let asset_manager = resolve_project_asset_manager(core)?;
    RenderService::spawn(asset_manager)
}

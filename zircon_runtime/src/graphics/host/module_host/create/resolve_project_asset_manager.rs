use std::sync::Arc;

use zircon_core::CoreHandle;
use zircon_graphics::GraphicsError;

use crate::asset::{pipeline::manager::ProjectAssetManager, PROJECT_ASSET_MANAGER_NAME};

pub(super) fn resolve_project_asset_manager(
    core: &CoreHandle,
) -> Result<Arc<ProjectAssetManager>, GraphicsError> {
    core.resolve_manager::<ProjectAssetManager>(PROJECT_ASSET_MANAGER_NAME)
        .map_err(|error| GraphicsError::Asset(error.to_string()))
}

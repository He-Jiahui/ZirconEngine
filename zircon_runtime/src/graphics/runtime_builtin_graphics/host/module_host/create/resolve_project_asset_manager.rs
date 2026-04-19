use std::sync::Arc;

use crate::core::CoreHandle;
use crate::asset::{pipeline::manager::ProjectAssetManager, PROJECT_ASSET_MANAGER_NAME};

use crate::graphics::GraphicsError;

pub(super) fn resolve_project_asset_manager(
    core: &CoreHandle,
) -> Result<Arc<ProjectAssetManager>, GraphicsError> {
    core.resolve_manager::<ProjectAssetManager>(PROJECT_ASSET_MANAGER_NAME)
        .map_err(|error| GraphicsError::Asset(error.to_string()))
}

use std::sync::Arc;

use zircon_asset::{ProjectAssetManager, PROJECT_ASSET_MANAGER_NAME};
use zircon_core::CoreHandle;

use crate::GraphicsError;

pub(super) fn resolve_project_asset_manager(
    core: &CoreHandle,
) -> Result<Arc<ProjectAssetManager>, GraphicsError> {
    core.resolve_manager::<ProjectAssetManager>(PROJECT_ASSET_MANAGER_NAME)
        .map_err(|error| GraphicsError::Asset(error.to_string()))
}

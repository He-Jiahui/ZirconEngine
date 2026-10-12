use crate::core::manager::{manager_service_handle, ManagerServiceHandle};
use crate::core::{CoreError, CoreHandle};

use super::super::PROJECT_ASSET_MANAGER_NAME;
use super::ProjectAssetManager;

pub fn project_asset_manager_handle(
    core: &CoreHandle,
) -> Result<ManagerServiceHandle<ProjectAssetManager>, CoreError> {
    manager_service_handle(core, PROJECT_ASSET_MANAGER_NAME)
}

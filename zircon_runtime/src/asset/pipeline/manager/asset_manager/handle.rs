use crate::core::manager::{manager_service_handle, ManagerServiceHandle};
use crate::core::{CoreError, CoreHandle};

use super::super::ASSET_MANAGER_NAME;
use super::AssetManager;

pub fn asset_manager_handle(
    core: &CoreHandle,
) -> Result<ManagerServiceHandle<dyn AssetManager>, CoreError> {
    manager_service_handle(core, ASSET_MANAGER_NAME)
}

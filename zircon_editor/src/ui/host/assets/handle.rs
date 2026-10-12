use zircon_runtime::core::manager::{manager_service_handle, ManagerServiceHandle};
use zircon_runtime::core::{CoreError, CoreHandle};

use crate::ui::host::module::EDITOR_ASSET_MANAGER_NAME;

use super::EditorAssetManager;

pub fn editor_asset_manager_handle(
    core: &CoreHandle,
) -> Result<ManagerServiceHandle<dyn EditorAssetManager>, CoreError> {
    manager_service_handle(core, EDITOR_ASSET_MANAGER_NAME)
}

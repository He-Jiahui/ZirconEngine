//! Read-only runtime resource boundary. Desktop authoring remains behind native-host.
use crate::core::CoreError;
pub(crate) fn asset_error_message(message: impl Into<String>) -> CoreError {
    CoreError::Initialization(
        crate::asset::PROJECT_ASSET_MANAGER_NAME.to_owned(),
        message.into(),
    )
}
#[cfg(feature = "native-host")]
pub use crate::asset::pipeline::manager::{
    project_asset_manager_handle as runtime_asset_manager_handle,
    ProjectAssetManager as RuntimeAssetManager,
    ProjectAssetManagerAccess as RuntimeAssetManagerAccess,
};
#[cfg(not(feature = "native-host"))]
#[path = "manager.rs"]
mod manager;
#[cfg(not(feature = "native-host"))]
pub use manager::{CookedAssetSource, RuntimeAssetManager};
#[cfg(not(feature = "native-host"))]
use RuntimeAssetManager as ProjectAssetManager;
#[cfg(not(feature = "native-host"))]
#[path = "../pipeline/manager/assets/access.rs"]
mod access;
#[cfg(not(feature = "native-host"))]
pub use access::ProjectAssetManagerAccess as RuntimeAssetManagerAccess;
#[cfg(not(feature = "native-host"))]
pub fn runtime_asset_manager_handle(
    core: &crate::core::CoreHandle,
) -> Result<crate::core::manager::ManagerServiceHandle<RuntimeAssetManager>, CoreError> {
    crate::core::manager::manager_service_handle(core, crate::asset::PROJECT_ASSET_MANAGER_NAME)
}
#[cfg(not(feature = "native-host"))]
#[path = "../pipeline/manager/assets/loading/acquire_asset.rs"]
mod acquire_asset;
#[cfg(not(feature = "native-host"))]
#[path = "../pipeline/manager/assets/loading/acquire_typed.rs"]
mod acquire_typed;
#[cfg(not(feature = "native-host"))]
#[path = "../pipeline/manager/assets/loading/load_asset.rs"]
mod load_asset;
#[cfg(not(feature = "native-host"))]
#[path = "../manager/assets/loading/load_effective_material.rs"]
mod load_effective_material;
#[cfg(not(feature = "native-host"))]
#[path = "../pipeline/manager/assets/loading/load_imported_asset.rs"]
mod load_imported_asset;
#[cfg(not(feature = "native-host"))]
#[path = "../pipeline/manager/assets/loading/load_typed.rs"]
mod load_typed;
#[cfg(not(feature = "native-host"))]
#[path = "../pipeline/manager/assets/loading/load_typed_snapshot.rs"]
mod load_typed_snapshot;

#[cfg(not(feature = "native-host"))]
#[path = "../manager/service_contracts/resource_manager_contract.rs"]
mod resource_manager_contract;

#[cfg(target_arch = "wasm32")]
#[path = "fetch.rs"]
pub(crate) mod fetch;

mod generation;
pub use generation::{RuntimeAssetGenerationToken, RuntimeGenerationMatch};

#[cfg(any(target_arch = "wasm32", test))]
mod async_value;
#[cfg(target_arch = "wasm32")]
pub mod browser;

#[cfg(not(feature = "native-host"))]
mod module;
#[cfg(not(feature = "native-host"))]
pub use module::module_descriptor;

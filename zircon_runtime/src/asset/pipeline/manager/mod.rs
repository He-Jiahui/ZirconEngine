//! Module wiring and high-level asset manager service.

mod asset_manager;
mod builtins;
mod driver;
mod errors;
mod project_asset_manager;
mod records;
mod registration;
mod resource_sync;
mod service_contracts;

pub use asset_manager::{AssetManager, asset_manager_handle};
pub use driver::AssetIoDriver;
pub use project_asset_manager::{
    ProjectAssetManager, ProjectAssetManagerAccess, ProjectAssetWatchDiagnostics,
    project_asset_manager_handle,
};
pub use records::{AssetPipelineInfo, AssetStatusRecord, ProjectInfo};
pub use registration::{ASSET_MANAGER_NAME, PROJECT_ASSET_MANAGER_NAME};

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

pub use asset_manager::{asset_manager_handle, AssetManager};
pub use driver::AssetIoDriver;
pub use project_asset_manager::{
    project_asset_manager_handle, ProjectAssetManager, ProjectAssetManagerAccess,
};
pub use records::{AssetPipelineInfo, AssetStatusRecord, ProjectInfo};
pub use registration::{ASSET_MANAGER_NAME, PROJECT_ASSET_MANAGER_NAME};

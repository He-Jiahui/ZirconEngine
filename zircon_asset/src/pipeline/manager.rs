//! Module wiring and high-level asset manager service.

mod asset_manager;
mod builtins;
mod driver;
mod errors;
mod facades;
mod project_asset_manager;
mod records;
mod registration;
mod resource_sync;

pub use asset_manager::{resolve_asset_manager, AssetManager, AssetManagerHandle};
pub use driver::AssetIoDriver;
pub use project_asset_manager::ProjectAssetManager;
pub use records::{AssetPipelineInfo, AssetStatusRecord, ProjectInfo};
pub use registration::{ASSET_MANAGER_NAME, EDITOR_ASSET_MANAGER_NAME, PROJECT_ASSET_MANAGER_NAME};

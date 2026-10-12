//! Module wiring and high-level asset manager service.

#[path = "asset_manager/mod.rs"]
mod asset_manager;
#[path = "builtins/mod.rs"]
mod builtins;
#[path = "driver/mod.rs"]
mod driver;
#[path = "errors/mod.rs"]
mod errors;
#[path = "assets/mod.rs"]
mod project_asset_manager;
#[path = "records/mod.rs"]
mod records;
#[path = "registration/mod.rs"]
mod registration;
#[path = "resource_sync/mod.rs"]
mod resource_sync;
#[path = "service_contracts/mod.rs"]
mod service_contracts;

pub use asset_manager::{asset_manager_handle, AssetManager};
pub use driver::AssetIoDriver;
pub use project_asset_manager::{
    project_asset_manager_handle, ProjectAssetGenerationSnapshot, ProjectAssetGenerationToken,
    ProjectAssetManager, ProjectAssetManagerAccess, ProjectAssetWatchDiagnostics,
    ProjectGeneratedSourceReceipt, ProjectGenerationCommitOutcome, ProjectGenerationMatch,
};
pub use records::{AssetPipelineInfo, AssetStatusRecord, ProjectInfo};
pub use registration::{ASSET_MANAGER_NAME, PROJECT_ASSET_MANAGER_NAME};

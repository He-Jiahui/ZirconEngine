//! Module wiring and high-level asset manager service.

mod asset_io_driver;
mod asset_manager_facade;
mod builtins;
mod errors;
mod module_descriptor;
mod project_asset_manager;
mod project_asset_manager_construction;
mod project_asset_manager_debug;
mod project_asset_manager_loading;
mod project_asset_manager_runtime;
mod records;
mod resource_manager_facade;
mod resource_sync;
mod service_names;

pub use asset_io_driver::AssetIoDriver;
pub use module_descriptor::module_descriptor;
pub use project_asset_manager::ProjectAssetManager;
pub use service_names::{
    ASSET_IO_DRIVER_NAME, ASSET_MANAGER_NAME, ASSET_MODULE_NAME, EDITOR_ASSET_MANAGER_NAME,
    PROJECT_ASSET_MANAGER_NAME, RESOURCE_MANAGER_NAME,
};

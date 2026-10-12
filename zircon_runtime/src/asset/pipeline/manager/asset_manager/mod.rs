#[path = "asset_manager.rs"]
mod asset_manager;
#[path = "handle.rs"]
mod handle;

pub use asset_manager::AssetManager;
pub use handle::asset_manager_handle;

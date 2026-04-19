//! Runtime asset module registration absorbed into the runtime layer.

mod module;

pub use module::{
    module_descriptor, AssetModule, ASSET_IO_DRIVER_NAME, ASSET_MANAGER_NAME, ASSET_MODULE_NAME,
    EDITOR_ASSET_MANAGER_NAME, PROJECT_ASSET_MANAGER_NAME, RESOURCE_MANAGER_NAME,
};
pub use zircon_asset::artifact;
pub use zircon_asset::assets;
pub use zircon_asset::editor;
pub use zircon_asset::importer;
pub use zircon_asset::pipeline;
pub use zircon_asset::project;
pub use zircon_asset::watch;

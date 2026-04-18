mod module_descriptor;
mod service_names;

pub use module_descriptor::module_descriptor;
pub use service_names::{
    ASSET_IO_DRIVER_NAME, ASSET_MANAGER_NAME, ASSET_MODULE_NAME, EDITOR_ASSET_MANAGER_NAME,
    PROJECT_ASSET_MANAGER_NAME, RESOURCE_MANAGER_NAME,
};

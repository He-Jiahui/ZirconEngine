mod config;
mod module;
mod service_types;

pub use config::TextureConfig;
pub use module::{
    module_descriptor, TextureModule, TEXTURE_DRIVER_NAME, TEXTURE_MANAGER_NAME,
    TEXTURE_MODULE_NAME,
};
pub use service_types::{TextureDriver, TextureManager};

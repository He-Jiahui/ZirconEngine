//! Platform/windowing integration absorbed into the runtime layer.

mod config;
mod module;
mod service_types;

pub use config::PlatformConfig;
pub use module::{
    module_descriptor, PlatformModule, PLATFORM_DRIVER_NAME, PLATFORM_MANAGER_NAME,
    PLATFORM_MODULE_NAME,
};
pub use service_types::{PlatformDriver, PlatformManager};

#[cfg(test)]
mod tests;

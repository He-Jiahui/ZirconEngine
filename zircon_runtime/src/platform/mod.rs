//! Platform/windowing integration absorbed into the runtime layer.

mod capability;
mod config;
mod feature_selection;
mod module;
mod service_types;
mod target;

pub use capability::{
    CapabilityStatus, EventLoopPolicy, FileDragDropBackend, GamepadBackend, InputBackend,
    LinuxWindowProtocol, PlatformCapabilityMatrix, PlatformCapabilityReport, WindowBackend,
};
pub use config::{PlatformConfig, PLATFORM_CONFIG_KEY};
pub use feature_selection::PlatformFeatureSelection;
pub use module::{
    module_descriptor, PlatformModule, PLATFORM_DRIVER_NAME, PLATFORM_MANAGER_NAME,
    PLATFORM_MODULE_NAME,
};
pub use service_types::{PlatformDriver, PlatformManager};
pub use target::PlatformTarget;

#[cfg(test)]
mod tests;

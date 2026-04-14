//! Platform module skeleton wired into the core runtime.

use zircon_module::{stub_module_descriptor, ModuleDescriptor};

pub const PLATFORM_MODULE_NAME: &str = "PlatformModule";

#[derive(Clone, Debug, Default)]
pub struct PlatformConfig {
    pub enabled: bool,
}

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
        PLATFORM_MODULE_NAME,
        "Platform, windowing, and OS integration",
        "PlatformDriver",
        "PlatformManager",
    )
}

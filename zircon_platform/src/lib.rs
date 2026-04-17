//! Platform module skeleton wired into the core runtime.

use zircon_module::{stub_module_descriptor, EngineModule, ModuleDescriptor};

pub const PLATFORM_MODULE_NAME: &str = "PlatformModule";

#[derive(Clone, Debug, Default)]
pub struct PlatformConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PlatformModule;

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
        PLATFORM_MODULE_NAME,
        "Platform, windowing, and OS integration",
        "PlatformDriver",
        "PlatformManager",
    )
}

impl EngineModule for PlatformModule {
    fn module_name(&self) -> &'static str {
        PLATFORM_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Platform, windowing, and OS integration"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}

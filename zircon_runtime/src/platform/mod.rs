//! Platform/windowing integration absorbed into the runtime layer.

use std::sync::Arc;

use crate::engine_module::{
    dependency_on, factory, qualified_name, DriverDescriptor, EngineModule, ManagerDescriptor,
    ModuleDescriptor, ServiceKind, StartupMode,
};

pub const PLATFORM_MODULE_NAME: &str = "PlatformModule";
pub const PLATFORM_DRIVER_NAME: &str = "PlatformModule.Driver.PlatformDriver";
pub const PLATFORM_MANAGER_NAME: &str = "PlatformModule.Manager.PlatformManager";

#[derive(Clone, Debug, Default)]
pub struct PlatformConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PlatformModule;

#[derive(Clone, Debug, Default)]
pub struct PlatformDriver;

#[derive(Clone, Debug, Default)]
pub struct PlatformManager;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        PLATFORM_MODULE_NAME,
        "Platform, windowing, and OS integration",
    )
    .with_driver(DriverDescriptor::new(
        qualified_name(PLATFORM_MODULE_NAME, ServiceKind::Driver, "PlatformDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(PlatformDriver::default()) as _)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            PLATFORM_MODULE_NAME,
            ServiceKind::Manager,
            "PlatformManager",
        ),
        StartupMode::Lazy,
        vec![dependency_on(
            PLATFORM_MODULE_NAME,
            ServiceKind::Driver,
            "PlatformDriver",
        )],
        factory(|_| Ok(Arc::new(PlatformManager::default()) as _)),
    ))
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

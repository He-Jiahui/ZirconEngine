use std::sync::Arc;

use super::{PlatformDriver, PlatformManager};
use crate::core::framework::foundation::FOUNDATION_MODULE_NAME;
use crate::core::framework::platform::{PreferenceStorage, PLATFORM_MODULE_NAME};
use crate::core::manager::RegisteredManagerService;
use crate::core::runtime::ServiceObject;
use crate::engine_module::{
    dependency_on, factory, qualified_name, DriverDescriptor, EngineModule, InitLevel,
    ManagerDescriptor, ModuleDependencySpec, ModuleDescriptor, ServiceKind, StartupMode,
};

pub const PLATFORM_DRIVER_NAME: &str = "PlatformModule.Driver.PlatformDriver";

#[derive(Clone, Copy, Debug, Default)]
pub struct PlatformModule;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        PLATFORM_MODULE_NAME,
        "Platform, windowing, and OS integration",
    )
    .with_init_level(InitLevel::Services)
    .with_module_dependency(ModuleDependencySpec::named(FOUNDATION_MODULE_NAME))
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
        factory(|core| {
            let driver = core.resolve_driver::<PlatformDriver>(PLATFORM_DRIVER_NAME)?;
            let manager = Arc::new(PlatformManager::new(driver));
            Ok(
                Arc::new(RegisteredManagerService::<dyn PreferenceStorage>::new(
                    manager,
                )) as ServiceObject,
            )
        }),
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

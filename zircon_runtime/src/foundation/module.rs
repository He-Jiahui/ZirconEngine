use std::sync::Arc;

use crate::core::manager::{ConfigManagerHandle, EventManagerHandle};
use crate::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use crate::engine_module::{factory, qualified_name, EngineModule};

use super::{ConfigDriver, DefaultConfigManager, DefaultEventManager, EventDriver};

pub const FOUNDATION_MODULE_NAME: &str = "FoundationModule";
pub const CONFIG_DRIVER_NAME: &str = "FoundationModule.Driver.ConfigDriver";
pub const EVENT_DRIVER_NAME: &str = "FoundationModule.Driver.EventDriver";

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        FOUNDATION_MODULE_NAME,
        "Built-in runtime foundation services",
    )
    .with_driver(DriverDescriptor::new(
        qualified_name(FOUNDATION_MODULE_NAME, ServiceKind::Driver, "ConfigDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(ConfigDriver) as ServiceObject)),
    ))
    .with_driver(DriverDescriptor::new(
        qualified_name(FOUNDATION_MODULE_NAME, ServiceKind::Driver, "EventDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(EventDriver) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            FOUNDATION_MODULE_NAME,
            ServiceKind::Manager,
            "ConfigManager",
        ),
        StartupMode::Immediate,
        Vec::new(),
        factory(|core| {
            let manager = Arc::new(DefaultConfigManager::new(core.clone()));
            Ok(Arc::new(ConfigManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(FOUNDATION_MODULE_NAME, ServiceKind::Manager, "EventManager"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|core| {
            let manager = Arc::new(DefaultEventManager::new(core.clone()));
            Ok(Arc::new(EventManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoundationModule;

impl EngineModule for FoundationModule {
    fn module_name(&self) -> &'static str {
        FOUNDATION_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Built-in runtime foundation services"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}

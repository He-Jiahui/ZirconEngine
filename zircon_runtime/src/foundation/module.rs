use std::sync::Arc;

use zircon_core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_manager::{ConfigManagerHandle, EventManagerHandle};
use zircon_module::{factory, qualified_name};

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

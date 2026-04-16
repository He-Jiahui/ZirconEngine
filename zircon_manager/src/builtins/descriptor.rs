use std::sync::Arc;

use zircon_core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_module::{factory, qualified_name};

use crate::{ConfigManagerHandle, EventManagerHandle};

use super::{
    names::MANAGER_MODULE_NAME, ConfigDriver, DefaultConfigManager, DefaultEventManager,
    EventDriver,
};

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(MANAGER_MODULE_NAME, "Stable manager facade layer")
        .with_driver(DriverDescriptor::new(
            qualified_name(MANAGER_MODULE_NAME, ServiceKind::Driver, "ConfigDriver"),
            StartupMode::Immediate,
            Vec::new(),
            factory(|_| Ok(Arc::new(ConfigDriver) as ServiceObject)),
        ))
        .with_driver(DriverDescriptor::new(
            qualified_name(MANAGER_MODULE_NAME, ServiceKind::Driver, "EventDriver"),
            StartupMode::Immediate,
            Vec::new(),
            factory(|_| Ok(Arc::new(EventDriver) as ServiceObject)),
        ))
        .with_manager(ManagerDescriptor::new(
            qualified_name(MANAGER_MODULE_NAME, ServiceKind::Manager, "ConfigManager"),
            StartupMode::Immediate,
            Vec::new(),
            factory(|core| {
                let manager = Arc::new(DefaultConfigManager::new(core.clone()));
                Ok(Arc::new(ConfigManagerHandle::new(manager)) as ServiceObject)
            }),
        ))
        .with_manager(ManagerDescriptor::new(
            qualified_name(MANAGER_MODULE_NAME, ServiceKind::Manager, "EventManager"),
            StartupMode::Immediate,
            Vec::new(),
            factory(|core| {
                let manager = Arc::new(DefaultEventManager::new(core.clone()));
                Ok(Arc::new(EventManagerHandle::new(manager)) as ServiceObject)
            }),
        ))
}

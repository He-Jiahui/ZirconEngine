use std::sync::Arc;

use zircon_core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_module::{dependency_on, factory, qualified_name, EngineModule};
use zircon_ui::event_ui::UiEventManager;

pub const UI_MODULE_NAME: &str = "UiModule";
pub const UI_RUNTIME_DRIVER_NAME: &str = "UiModule.Driver.UiRuntimeDriver";
pub const UI_EVENT_MANAGER_NAME: &str = "UiModule.Manager.UiEventManager";

#[derive(Clone, Debug, Default)]
pub struct UiConfig {
    pub enabled: bool,
}

#[derive(Debug, Default)]
pub struct UiRuntimeDriver;

#[derive(Clone, Copy, Debug, Default)]
pub struct UiModule;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(UI_MODULE_NAME, "Runtime UI widgets and layout")
        .with_driver(DriverDescriptor::new(
            qualified_name(UI_MODULE_NAME, ServiceKind::Driver, "UiRuntimeDriver"),
            StartupMode::Immediate,
            Vec::new(),
            factory(|_| Ok(Arc::new(UiRuntimeDriver) as ServiceObject)),
        ))
        .with_manager(ManagerDescriptor::new(
            qualified_name(UI_MODULE_NAME, ServiceKind::Manager, "UiEventManager"),
            StartupMode::Immediate,
            vec![dependency_on(
                UI_MODULE_NAME,
                ServiceKind::Driver,
                "UiRuntimeDriver",
            )],
            factory(|_| Ok(Arc::new(UiEventManager::default()) as ServiceObject)),
        ))
}

impl EngineModule for UiModule {
    fn module_name(&self) -> &'static str {
        UI_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Runtime UI widgets and layout"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}

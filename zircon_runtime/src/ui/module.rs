use std::sync::Arc;

use crate::core::framework::input::INPUT_MODULE_NAME;
use crate::core::framework::render::GRAPHICS_MODULE_NAME;
use crate::core::framework::scene::SCENE_MODULE_NAME;
use crate::core::framework::ui::UI_MODULE_NAME;
use crate::core::runtime::ServiceObject;
use crate::core::{
    DriverDescriptor, InitLevel, ManagerDescriptor, ModuleDependencySpec, ModuleDescriptor,
    ServiceKind, StartupMode,
};
use crate::engine_module::{dependency_on, factory, qualified_name, EngineModule};
use crate::ui::event_ui::UiEventManager;

mod lifecycle;
mod runtime_driver;

pub use runtime_driver::{UiConfig, UiRuntimeDriver, UI_CONFIG_KEY};

#[cfg(test)]
#[path = "module/tests/cases.rs"]
mod tests;

pub const UI_RUNTIME_DRIVER_NAME: &str = "UiModule.Driver.UiRuntimeDriver";
pub const UI_EVENT_MANAGER_NAME: &str = "UiModule.Manager.UiEventManager";

#[derive(Clone, Copy, Debug, Default)]
pub struct UiModule;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(UI_MODULE_NAME, "Runtime UI widgets and layout")
        .with_init_level(InitLevel::Scene)
        .with_lifecycle(Arc::new(lifecycle::UiModuleLifecycle))
        .with_module_dependency(ModuleDependencySpec::named(INPUT_MODULE_NAME))
        .with_module_dependency(ModuleDependencySpec::named(SCENE_MODULE_NAME))
        .with_module_dependency(ModuleDependencySpec::named(GRAPHICS_MODULE_NAME))
        .with_driver(DriverDescriptor::new(
            qualified_name(UI_MODULE_NAME, ServiceKind::Driver, "UiRuntimeDriver"),
            StartupMode::Immediate,
            Vec::new(),
            factory(|core| {
                let core = core
                    .upgrade()
                    .ok_or(crate::core::CoreError::RuntimeUnavailable)?;
                Ok(Arc::new(UiRuntimeDriver::from_core(&core)?) as ServiceObject)
            }),
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

use std::sync::Arc;

use crate::core::framework::input::INPUT_MODULE_NAME;
use crate::core::framework::input::{InputActionManager, InputManager};
use crate::core::framework::platform::PLATFORM_MODULE_NAME;
use crate::core::manager::RegisteredManagerService;
use crate::core::runtime::ServiceObject;
use crate::core::{
    DriverDescriptor, InitLevel, ManagerDescriptor, ModuleDependencySpec, ModuleDescriptor,
    ServiceKind, StartupMode,
};
use crate::engine_module::{factory, qualified_name};

use super::super::runtime::{DefaultInputManager, InputDriver};
use super::InputConfig;

pub const INPUT_DRIVER_NAME: &str = "InputModule.Driver.InputDriver";
pub const INPUT_MANAGER_NAME: &str = crate::core::manager::INPUT_MANAGER_NAME;
pub const INPUT_ACTION_MANAGER_NAME: &str = crate::core::manager::INPUT_ACTION_MANAGER_NAME;

pub fn module_descriptor() -> ModuleDescriptor {
    module_descriptor_with_config(InputConfig::default())
}

pub fn module_descriptor_with_config(config: InputConfig) -> ModuleDescriptor {
    let action_config = config;
    ModuleDescriptor::new(
        INPUT_MODULE_NAME,
        "High-level input routing and action maps",
    )
    .with_init_level(InitLevel::Services)
    .with_module_dependency(ModuleDependencySpec::named(PLATFORM_MODULE_NAME))
    .with_driver(DriverDescriptor::new(
        qualified_name(INPUT_MODULE_NAME, ServiceKind::Driver, "InputDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(InputDriver) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(INPUT_MODULE_NAME, ServiceKind::Manager, "InputManager"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| {
            let manager = Arc::new(DefaultInputManager::default());
            Ok(
                Arc::new(RegisteredManagerService::<dyn InputManager>::new(manager))
                    as ServiceObject,
            )
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            INPUT_MODULE_NAME,
            ServiceKind::Manager,
            "InputActionManager",
        ),
        StartupMode::Immediate,
        Vec::new(),
        factory(move |_| {
            let manager = Arc::new(action_config.action_manager());
            Ok(
                Arc::new(RegisteredManagerService::<dyn InputActionManager>::new(
                    manager,
                )) as ServiceObject,
            )
        }),
    ))
}

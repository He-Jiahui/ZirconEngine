use std::sync::Arc;

use crate::core::manager::{InputActionManagerHandle, InputManagerHandle};
use crate::core::runtime::ServiceObject;
use crate::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, StartupMode,
};
use crate::engine_module::{factory, qualified_name};

use super::super::runtime::{DefaultInputManager, InputDriver};
use super::InputConfig;

pub const INPUT_MODULE_NAME: &str = "InputModule";
pub const INPUT_DRIVER_NAME: &str = "InputModule.Driver.InputDriver";
pub const INPUT_MANAGER_NAME: &str = crate::core::manager::INPUT_MANAGER_NAME;
pub const INPUT_ACTION_MANAGER_NAME: &str = crate::core::manager::INPUT_ACTION_MANAGER_NAME;

pub fn module_descriptor() -> ModuleDescriptor {
    module_descriptor_with_config(InputConfig::default())
}

pub fn module_descriptor_with_config(config: InputConfig) -> ModuleDescriptor {
    let action_config = config.clone();
    ModuleDescriptor::new(
        INPUT_MODULE_NAME,
        "High-level input routing and action maps",
    )
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
            Ok(Arc::new(InputManagerHandle::new(manager)) as ServiceObject)
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
            Ok(Arc::new(InputActionManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
}

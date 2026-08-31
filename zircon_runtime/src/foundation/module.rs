use std::sync::Arc;

use crate::core::framework::foundation::{ConfigManager, EventManager, FOUNDATION_MODULE_NAME};
use crate::core::manager::RegisteredManagerService;
use crate::core::runtime::ServiceObject;
use crate::core::{
    CoreError, InitLevel, ManagerDescriptor, ModuleDescriptor, ServiceKind, StartupMode,
};
use crate::engine_module::{factory, qualified_name, EngineModule};

use super::{DefaultConfigManager, DefaultEventManager};

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        FOUNDATION_MODULE_NAME,
        "Built-in runtime foundation services",
    )
    .with_init_level(InitLevel::Kernel)
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            FOUNDATION_MODULE_NAME,
            ServiceKind::Manager,
            "ConfigManager",
        ),
        StartupMode::Immediate,
        Vec::new(),
        factory(|core| {
            let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
            let manager = Arc::new(DefaultConfigManager::new(&core)?);
            Ok(
                Arc::new(RegisteredManagerService::<dyn ConfigManager>::new(manager))
                    as ServiceObject,
            )
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(FOUNDATION_MODULE_NAME, ServiceKind::Manager, "EventManager"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|core| {
            let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
            let manager = Arc::new(DefaultEventManager::new(&core));
            Ok(
                Arc::new(RegisteredManagerService::<dyn EventManager>::new(manager))
                    as ServiceObject,
            )
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

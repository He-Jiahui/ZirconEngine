use std::sync::Arc;

use zircon_runtime::core::manager::AiManagerHandle;
use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, StartupMode,
};
use zircon_runtime::engine_module::{dependency_on, factory, qualified_name, EngineModule};

use crate::DefaultAiManager;

pub const AI_MODULE_NAME: &str = "ai.runtime";
pub const AI_DRIVER_NAME: &str = "ai.runtime.Driver.AiDriver";
pub(crate) const DEFAULT_AI_MANAGER_NAME: &str = "ai.runtime.Manager.DefaultAiManager";
pub const AI_MANAGER_NAME: &str = zircon_runtime::core::manager::AI_MANAGER_NAME;

#[derive(Clone, Debug, Default)]
pub struct AiDriver;

#[derive(Clone, Copy, Debug, Default)]
pub struct AiModule;

pub fn module_descriptor() -> ModuleDescriptor {
    module_descriptor_with_manager(None)
}

pub fn module_descriptor_with_manager(manager: Option<Arc<DefaultAiManager>>) -> ModuleDescriptor {
    let default_manager = manager.clone();
    ModuleDescriptor::new(
        AI_MODULE_NAME,
        "Behavior tree, blackboard, perception, and agent tick runtime contracts",
    )
    .with_driver(DriverDescriptor::new(
        qualified_name(AI_MODULE_NAME, ServiceKind::Driver, "AiDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(AiDriver) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(AI_MODULE_NAME, ServiceKind::Manager, "DefaultAiManager"),
        StartupMode::Lazy,
        vec![dependency_on(
            AI_MODULE_NAME,
            ServiceKind::Driver,
            "AiDriver",
        )],
        factory(move |_| {
            Ok(default_manager
                .clone()
                .unwrap_or_else(|| Arc::new(DefaultAiManager::default()))
                as ServiceObject)
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(AI_MODULE_NAME, ServiceKind::Manager, "AiManager"),
        StartupMode::Lazy,
        vec![dependency_on(
            AI_MODULE_NAME,
            ServiceKind::Manager,
            "DefaultAiManager",
        )],
        factory(move |core| {
            let manager = match &manager {
                Some(manager) => Arc::clone(manager),
                None => core.resolve_manager::<DefaultAiManager>(DEFAULT_AI_MANAGER_NAME)?,
            };
            Ok(Arc::new(AiManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
}

impl EngineModule for AiModule {
    fn module_name(&self) -> &'static str {
        AI_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Behavior tree, blackboard, perception, and agent tick runtime contracts"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}

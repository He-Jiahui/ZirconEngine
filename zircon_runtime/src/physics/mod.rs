mod physics_interface;
mod service_types;

use std::sync::Arc;

use crate::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use crate::core::manager::PhysicsManagerHandle;
use crate::engine_module::{dependency_on, factory, qualified_name, EngineModule};

pub use physics_interface::PhysicsInterface;
pub use service_types::{DefaultPhysicsManager, PhysicsDriver, JOLT_ENABLED};

pub const PHYSICS_MODULE_NAME: &str = "PhysicsModule";
pub const PHYSICS_DRIVER_NAME: &str = "PhysicsModule.Driver.PhysicsDriver";
const DEFAULT_PHYSICS_MANAGER_NAME: &str = "PhysicsModule.Manager.DefaultPhysicsManager";
pub const PHYSICS_MANAGER_NAME: &str = crate::core::manager::PHYSICS_MANAGER_NAME;

#[derive(Clone, Debug, Default)]
pub struct PhysicsConfig {
    pub enabled: bool,
    pub backend: &'static str,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PhysicsModule;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        PHYSICS_MODULE_NAME,
        "Physics world, queries, and backend selection",
    )
    .with_driver(DriverDescriptor::new(
        qualified_name(PHYSICS_MODULE_NAME, ServiceKind::Driver, "PhysicsDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(PhysicsDriver) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            PHYSICS_MODULE_NAME,
            ServiceKind::Manager,
            "DefaultPhysicsManager",
        ),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(DefaultPhysicsManager::default()) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(PHYSICS_MODULE_NAME, ServiceKind::Manager, "PhysicsManager"),
        StartupMode::Immediate,
        vec![dependency_on(
            PHYSICS_MODULE_NAME,
            ServiceKind::Manager,
            "DefaultPhysicsManager",
        )],
        factory(|core| {
            let manager =
                core.resolve_manager::<DefaultPhysicsManager>(DEFAULT_PHYSICS_MANAGER_NAME)?;
            Ok(Arc::new(PhysicsManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
}

impl EngineModule for PhysicsModule {
    fn module_name(&self) -> &'static str {
        PHYSICS_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Physics world, queries, and backend selection"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}

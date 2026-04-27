use std::sync::Arc;

use zircon_runtime::core::manager::PhysicsManagerHandle;
use zircon_runtime::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_runtime::engine_module::{dependency_on, factory, qualified_name, EngineModule};

use super::{DefaultPhysicsManager, PhysicsDriver};

pub const PHYSICS_MODULE_NAME: &str = "PhysicsModule";
pub const PHYSICS_DRIVER_NAME: &str = "PhysicsModule.Driver.PhysicsDriver";
pub(crate) const DEFAULT_PHYSICS_MANAGER_NAME: &str = "PhysicsModule.Manager.DefaultPhysicsManager";
pub const PHYSICS_MANAGER_NAME: &str = zircon_runtime::core::manager::PHYSICS_MANAGER_NAME;
pub const PHYSICS_SETTINGS_CONFIG_KEY: &str = "physics.settings";

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
        factory(|core| {
            Ok(Arc::new(DefaultPhysicsManager::new(Some(core.clone()))) as ServiceObject)
        }),
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

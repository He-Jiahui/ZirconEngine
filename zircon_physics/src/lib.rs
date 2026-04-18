//! Physics module scaffold with explicit core service descriptors.

use std::sync::Arc;

use zircon_module::{
    dependency_on, factory, qualified_name, DriverDescriptor, EngineModule, ManagerDescriptor,
    ModuleDescriptor, ServiceKind, StartupMode,
};

pub const JOLT_ENABLED: bool = cfg!(feature = "jolt");
pub const PHYSICS_MODULE_NAME: &str = "PhysicsModule";
pub const PHYSICS_DRIVER_NAME: &str = "PhysicsModule.Driver.PhysicsDriver";
pub const PHYSICS_MANAGER_NAME: &str = "PhysicsModule.Manager.PhysicsManager";

#[derive(Clone, Debug, Default)]
pub struct PhysicsConfig {
    pub enabled: bool,
    pub backend: &'static str,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PhysicsModule;

#[derive(Clone, Debug, Default)]
pub struct PhysicsDriver;

#[derive(Clone, Debug, Default)]
pub struct PhysicsManager;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        PHYSICS_MODULE_NAME,
        "Physics world, queries, and backend selection",
    )
    .with_driver(DriverDescriptor::new(
        qualified_name(PHYSICS_MODULE_NAME, ServiceKind::Driver, "PhysicsDriver"),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(PhysicsDriver::default()) as _)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(PHYSICS_MODULE_NAME, ServiceKind::Manager, "PhysicsManager"),
        StartupMode::Lazy,
        vec![dependency_on(
            PHYSICS_MODULE_NAME,
            ServiceKind::Driver,
            "PhysicsDriver",
        )],
        factory(|_| Ok(Arc::new(PhysicsManager::default()) as _)),
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

use zircon_module::{EngineModule, ModuleDescriptor};

pub use zircon_physics::{PhysicsDriver, PhysicsManager, JOLT_ENABLED};

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

pub fn module_descriptor() -> ModuleDescriptor {
    super::module_descriptor_with_driver_and_manager::<PhysicsDriver, PhysicsManager>(
        PHYSICS_MODULE_NAME,
        "Physics world, queries, and backend selection",
        "PhysicsDriver",
        "PhysicsManager",
    )
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

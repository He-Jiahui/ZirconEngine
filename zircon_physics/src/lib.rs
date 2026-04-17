//! Physics module skeleton wired into the core runtime.

use zircon_module::{stub_module_descriptor, EngineModule, ModuleDescriptor};

pub const JOLT_ENABLED: bool = cfg!(feature = "jolt");
pub const PHYSICS_MODULE_NAME: &str = "PhysicsModule";

#[derive(Clone, Debug, Default)]
pub struct PhysicsConfig {
    pub enabled: bool,
    pub backend: &'static str,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PhysicsModule;

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
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

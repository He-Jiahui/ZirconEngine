//! Physics module skeleton wired into the core runtime.

use zircon_module::{stub_module_descriptor, ModuleDescriptor};

pub const JOLT_ENABLED: bool = cfg!(feature = "jolt");
pub const PHYSICS_MODULE_NAME: &str = "PhysicsModule";

#[derive(Clone, Debug, Default)]
pub struct PhysicsConfig {
    pub enabled: bool,
    pub backend: &'static str,
}

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
        PHYSICS_MODULE_NAME,
        "Physics world, queries, and backend selection",
        "PhysicsDriver",
        "PhysicsManager",
    )
}

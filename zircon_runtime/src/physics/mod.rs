//! Shared physics runtime contract, manager implementation, and scene sync helpers.

mod module;
pub mod runtime;

pub use module::{
    module_descriptor, PhysicsModule, DEFAULT_PHYSICS_MANAGER_NAME, PHYSICS_DRIVER_NAME,
    PHYSICS_MANAGER_NAME, PHYSICS_MODULE_NAME, PHYSICS_SETTINGS_CONFIG_KEY,
};
pub use runtime::{
    build_world_sync_state, integrate_builtin_physics_steps, DefaultPhysicsManager, PhysicsDriver,
    PhysicsTickPlan, JOLT_ENABLED,
};

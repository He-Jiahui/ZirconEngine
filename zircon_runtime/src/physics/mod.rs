mod config;
mod module;
mod physics_interface;
mod service_types;

pub use config::PhysicsConfig;
pub use module::{
    module_descriptor, PhysicsModule, PHYSICS_DRIVER_NAME, PHYSICS_MANAGER_NAME,
    PHYSICS_MODULE_NAME, PHYSICS_SETTINGS_CONFIG_KEY,
};
pub use physics_interface::PhysicsInterface;
pub use service_types::{
    build_world_sync_state, integrate_builtin_physics_steps, DefaultPhysicsManager, PhysicsDriver,
    PhysicsTickPlan, JOLT_ENABLED,
};

#[cfg(test)]
pub(crate) use module::DEFAULT_PHYSICS_MANAGER_NAME;

#[cfg(test)]
mod tests;

pub const PLUGIN_ID: &str = "physics";
pub const PHYSICS_SETTINGS_CONFIG_KEY: &str = "physics.settings";

mod backend;
mod capability;
mod manager;
mod module;
mod plugin;
mod query_contact;
mod runtime_system;
mod trigger;

pub use backend::JOLT_ENABLED;
pub use capability::{
    PHYSICS_CONSTRAINTS_CAPABILITY, PHYSICS_OVERLAP_CAPABILITY, PHYSICS_RAYCAST_CAPABILITY,
    PHYSICS_RUNTIME_CAPABILITY, PHYSICS_SHAPE_CAST_CAPABILITY, PHYSICS_SKELETAL_JOINTS_CAPABILITY,
    PHYSICS_TRIGGER_EVENTS_CAPABILITY, RUNTIME_CAPABILITIES,
};
pub use manager::{
    build_world_sync_state, integrate_builtin_physics_steps, DefaultPhysicsManager, PhysicsTickPlan,
};
pub use module::{
    module_descriptor, PhysicsDriver, PhysicsModule, DEFAULT_PHYSICS_MANAGER_NAME,
    PHYSICS_DRIVER_NAME, PHYSICS_MODULE_NAME,
};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, PhysicsRuntimePlugin, PHYSICS_DIST_CRATE_NAME,
    PHYSICS_DIST_RUNTIME_ENTRY, PLUGIN_RUNTIME_MODULE_NAME,
};
pub use runtime_system::{
    register_runtime_system, PhysicsRuntimeSystem, PHYSICS_STEP_SYSTEM, PHYSICS_SYSTEM_SET,
};
pub use zircon_runtime::core::manager::PHYSICS_MANAGER_NAME;

#[cfg(test)]
mod tests;

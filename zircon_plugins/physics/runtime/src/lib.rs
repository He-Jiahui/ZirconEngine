pub const PHYSICS_SETTINGS_CONFIG_KEY: &str = "physics.settings";

mod backend;
mod capability;
mod constraint;
mod diagnostics;
mod manager;
mod module;
mod plugin;
mod runtime_system;
mod skeletal;

pub use backend::builtin::integrate_builtin_physics_steps;
#[cfg(feature = "backend-jolt")]
pub use backend::JoltPhysicsBackend;
pub use backend::{
    BodyCommand, BodyDesc, BodyHandle, BuiltinPhysicsBackend, ConstraintDesc, ConstraintHandle,
    PhysicsBackend, PhysicsBackendError, PhysicsBackendObjectKind, PhysicsEventBuffer, ShapeHandle,
    JOLT_ENABLED,
};
pub use capability::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, PHYSICS_CONSTRAINTS_CAPABILITY, PHYSICS_DECLARATION,
    PHYSICS_OVERLAP_CAPABILITY, PHYSICS_RAYCAST_CAPABILITY, PHYSICS_RUNTIME_CAPABILITY,
    PHYSICS_SHAPE_CAST_CAPABILITY, PHYSICS_SKELETAL_JOINTS_CAPABILITY,
    PHYSICS_TRIGGER_EVENTS_CAPABILITY, PLUGIN_ID, RUNTIME_CAPABILITIES,
};
pub use constraint::{AxisConstraint, JointParams, JointSpring};
pub use diagnostics::{record_physics_step_diagnostic, PHYSICS_STEP_DURATION_DIAGNOSTIC_PATH};
pub use manager::{
    build_world_sync_state, DefaultPhysicsManager, PhysicsBodyCommand, PhysicsCommandError,
    PhysicsTickPlan,
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
    register_runtime_systems, PhysicsRuntimeSystem, PHYSICS_CONTACT_EVENT_ID,
    PHYSICS_CONTACT_EVENT_SCHEMA, PHYSICS_STEP_SYSTEM, PHYSICS_SYNC_TO_SCENE_SYSTEM,
    PHYSICS_SYSTEM_SET, PHYSICS_TRIGGER_EVENT_ID, PHYSICS_TRIGGER_EVENT_SCHEMA,
};
pub use skeletal::{
    RagdollBoneProfile, RagdollMode, RagdollProfile, RagdollProfileError, RagdollRuntime,
    RagdollSpawn,
};
pub use zircon_runtime::core::framework::physics::{
    PhysicsQueryInterface, PHYSICS_QUERY_INTERFACE_ID,
};
pub use zircon_runtime::core::manager::PHYSICS_MANAGER_NAME;

#[cfg(test)]
mod tests;

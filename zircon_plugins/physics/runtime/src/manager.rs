use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg(any(feature = "backend-jolt", test))]
#[path = "manager/change_detection.rs"]
mod change_detection;
#[path = "manager/clock.rs"]
mod clock;
#[path = "manager/command_buffer.rs"]
mod command_buffer;
#[cfg(feature = "backend-jolt")]
#[path = "manager/jolt_world.rs"]
mod jolt_world;
#[path = "manager/poison_recovery.rs"]
mod poison_recovery;
#[path = "manager/query.rs"]
mod query;
#[path = "manager/service.rs"]
mod service;
#[path = "manager/settings.rs"]
mod settings;
#[path = "manager/validation.rs"]
pub(crate) mod validation;
#[path = "manager/world_sync.rs"]
mod world_sync;

#[cfg(test)]
#[path = "manager/tests/cases.rs"]
mod tests;

use zircon_runtime::core::framework::foundation::ConfigManager;
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::framework::{
    physics::{
        PhysicsContactEvent, PhysicsSettings, PhysicsTriggerEvent, PhysicsWorldStepPlan,
        PhysicsWorldSyncState,
    },
    scene::physics::PhysicsMaterialMetadata,
};
use zircon_runtime::core::manager::ManagerServiceHandle;
use zircon_runtime::core::CoreWeak;

use crate::backend::builtin::PhysicsTriggerPairMap;

pub use command_buffer::{PhysicsBodyCommand, PhysicsCommandError};
pub(crate) use world_sync::apply_synchronized_bodies_to_scene;
pub use world_sync::build_world_sync_state;

pub type PhysicsTickPlan = PhysicsWorldStepPlan;

#[derive(Clone, Debug)]
pub struct DefaultPhysicsManager {
    // Shared managers may be registry-owned; retain only a weak runtime attachment.
    core: Arc<Mutex<Option<CoreWeak>>>,
    config_manager: Arc<Mutex<Option<ManagerServiceHandle<dyn ConfigManager>>>>,
    settings: Arc<Mutex<PhysicsSettings>>,
    default_material: PhysicsMaterialMetadata,
    accumulators: Arc<Mutex<HashMap<WorldHandle, f32>>>,
    synced_worlds: Arc<Mutex<HashMap<WorldHandle, Arc<PhysicsWorldSyncState>>>>,
    contacts: Arc<Mutex<HashMap<WorldHandle, Vec<PhysicsContactEvent>>>>,
    trigger_pairs: Arc<Mutex<HashMap<WorldHandle, PhysicsTriggerPairMap>>>,
    triggers: Arc<Mutex<HashMap<WorldHandle, Vec<PhysicsTriggerEvent>>>>,
    body_commands: Arc<Mutex<command_buffer::PhysicsBodyCommandQueues>>,
    last_backend_error: Arc<Mutex<Option<String>>>,
    #[cfg(feature = "backend-jolt")]
    jolt_worlds: Arc<Mutex<HashMap<WorldHandle, jolt_world::JoltManagedWorld>>>,
}

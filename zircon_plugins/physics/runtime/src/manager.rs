use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod builtin_step;
mod clock;
mod query;
mod service;
mod settings;
mod validation;
mod world_sync;

use zircon_runtime::core::framework::physics::{
    PhysicsContactEvent, PhysicsMaterialMetadata, PhysicsSettings, PhysicsTriggerEvent,
    PhysicsWorldStepPlan, PhysicsWorldSyncState,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::CoreHandle;

use crate::trigger::PhysicsTriggerPairMap;

pub use builtin_step::integrate_builtin_physics_steps;
pub use world_sync::build_world_sync_state;

pub type PhysicsTickPlan = PhysicsWorldStepPlan;

#[derive(Clone, Debug)]
pub struct DefaultPhysicsManager {
    core: Arc<Mutex<Option<CoreHandle>>>,
    settings: Arc<Mutex<PhysicsSettings>>,
    default_material: PhysicsMaterialMetadata,
    accumulators: Arc<Mutex<HashMap<WorldHandle, f32>>>,
    synced_worlds: Arc<Mutex<HashMap<WorldHandle, PhysicsWorldSyncState>>>,
    contacts: Arc<Mutex<HashMap<WorldHandle, Vec<PhysicsContactEvent>>>>,
    trigger_pairs: Arc<Mutex<HashMap<WorldHandle, PhysicsTriggerPairMap>>>,
    triggers: Arc<Mutex<HashMap<WorldHandle, Vec<PhysicsTriggerEvent>>>>,
}

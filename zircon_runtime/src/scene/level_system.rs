//! Runtime level instance wrapping one ECS world plus lifecycle metadata.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::core::framework::animation::AnimationPoseOutput;
use crate::core::framework::physics::{PhysicsContactEvent, PhysicsWorldStepPlan};
use crate::core::framework::scene::WorldHandle;
use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle};
use crate::scene::world::World;
use crate::scene::{EntityId, WorldDriver, WORLD_DRIVER_NAME};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LevelLifecycleState {
    Loaded,
    Unloaded,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LevelMetadata {
    pub project_root: Option<String>,
    pub asset_uri: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Clone)]
pub struct LevelSystem {
    handle: WorldHandle,
    inner: Arc<Mutex<World>>,
    runtime_state: Arc<Mutex<WorldRuntimeState>>,
    metadata: Arc<Mutex<LevelMetadata>>,
    lifecycle: Arc<Mutex<LevelLifecycleState>>,
    subsystems: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone, Debug, Default)]
struct WorldRuntimeState {
    physics_step_plan: Option<PhysicsWorldStepPlan>,
    physics_contacts: Vec<PhysicsContactEvent>,
    animation_poses: BTreeMap<EntityId, AnimationPoseOutput>,
    animation_graph_times: BTreeMap<EntityId, Real>,
    animation_state_machine_times: BTreeMap<EntityId, Real>,
}

impl LevelSystem {
    pub(crate) fn new(
        handle: WorldHandle,
        inner: Arc<Mutex<World>>,
        metadata: LevelMetadata,
    ) -> Self {
        Self {
            handle,
            inner,
            runtime_state: Arc::new(Mutex::new(WorldRuntimeState::default())),
            metadata: Arc::new(Mutex::new(metadata)),
            lifecycle: Arc::new(Mutex::new(LevelLifecycleState::Loaded)),
            subsystems: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn handle(&self) -> WorldHandle {
        self.handle
    }

    pub fn world_handle(&self) -> WorldHandle {
        self.handle
    }

    pub fn snapshot(&self) -> World {
        self.inner.lock().unwrap().clone()
    }

    pub fn replace(&self, world: World) {
        *self.inner.lock().unwrap() = world;
    }

    pub fn with_world<R>(&self, read: impl FnOnce(&World) -> R) -> R {
        let world = self.inner.lock().unwrap();
        read(&world)
    }

    pub fn with_world_mut<R>(&self, write: impl FnOnce(&mut World) -> R) -> R {
        let mut world = self.inner.lock().unwrap();
        write(&mut world)
    }

    pub fn tick(&self, core: &CoreHandle, delta_seconds: Real) -> Result<(), CoreError> {
        let driver = core.resolve_driver::<WorldDriver>(WORLD_DRIVER_NAME)?;
        driver.tick_level(core, self, delta_seconds)
    }

    pub fn last_physics_step_plan(&self) -> Option<PhysicsWorldStepPlan> {
        self.runtime_state.lock().unwrap().physics_step_plan
    }

    pub fn physics_contacts(&self) -> Vec<PhysicsContactEvent> {
        self.runtime_state.lock().unwrap().physics_contacts.clone()
    }

    pub fn animation_pose(&self, entity: EntityId) -> Option<AnimationPoseOutput> {
        self.runtime_state
            .lock()
            .unwrap()
            .animation_poses
            .get(&entity)
            .cloned()
    }

    pub(crate) fn animation_poses(&self) -> BTreeMap<EntityId, AnimationPoseOutput> {
        self.runtime_state.lock().unwrap().animation_poses.clone()
    }

    pub(crate) fn animation_playback_times(
        &self,
    ) -> (BTreeMap<EntityId, Real>, BTreeMap<EntityId, Real>) {
        let runtime_state = self.runtime_state.lock().unwrap();
        (
            runtime_state.animation_graph_times.clone(),
            runtime_state.animation_state_machine_times.clone(),
        )
    }

    pub(crate) fn record_animation_poses(
        &self,
        animation_poses: BTreeMap<EntityId, AnimationPoseOutput>,
    ) {
        self.runtime_state.lock().unwrap().animation_poses = animation_poses;
    }

    pub(crate) fn record_animation_playback_times(
        &self,
        animation_graph_times: BTreeMap<EntityId, Real>,
        animation_state_machine_times: BTreeMap<EntityId, Real>,
    ) {
        let mut runtime_state = self.runtime_state.lock().unwrap();
        runtime_state.animation_graph_times = animation_graph_times;
        runtime_state.animation_state_machine_times = animation_state_machine_times;
    }

    pub fn metadata(&self) -> LevelMetadata {
        self.metadata.lock().unwrap().clone()
    }

    pub fn set_metadata(&self, metadata: LevelMetadata) {
        *self.metadata.lock().unwrap() = metadata;
    }

    pub fn lifecycle(&self) -> LevelLifecycleState {
        self.lifecycle.lock().unwrap().clone()
    }

    pub fn set_lifecycle(&self, lifecycle: LevelLifecycleState) {
        *self.lifecycle.lock().unwrap() = lifecycle;
    }

    pub fn register_subsystem(&self, subsystem_name: impl Into<String>) {
        self.subsystems.lock().unwrap().push(subsystem_name.into());
    }

    pub fn registered_subsystems(&self) -> Vec<String> {
        self.subsystems.lock().unwrap().clone()
    }
}

impl std::fmt::Debug for LevelSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LevelSystem")
            .field("handle", &self.handle)
            .field("metadata", &self.metadata())
            .field("lifecycle", &self.lifecycle())
            .finish()
    }
}

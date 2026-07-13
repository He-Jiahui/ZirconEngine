//! Runtime level instance wrapping one ECS world plus lifecycle metadata.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::framework::animation::AnimationPoseOutput;
use crate::core::framework::scene::WorldHandle;
use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle, RuntimeTimeAdvance};
use crate::scene::world::World;
use crate::scene::{ecs::RuntimeSceneSystemContext, EntityId, WorldDriver, WORLD_DRIVER_NAME};

#[cfg(feature = "physics-contracts")]
#[path = "level_system/physics_runtime_enabled.rs"]
mod physics_runtime;
#[cfg(not(feature = "physics-contracts"))]
#[path = "level_system/physics_runtime_disabled.rs"]
mod physics_runtime;

use physics_runtime::PhysicsRuntimeState;

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
    physics: PhysicsRuntimeState,
    animation_poses: BTreeMap<EntityId, AnimationPoseOutput>,
    animation_graph_times: BTreeMap<EntityId, Real>,
    animation_state_machine_times: BTreeMap<EntityId, Real>,
    animation_state_machine_transitions: BTreeMap<EntityId, AnimationStateTransitionRuntime>,
    script_started_bindings: BTreeSet<(EntityId, String)>,
}

#[derive(Clone, Debug)]
pub struct AnimationStateTransitionRuntime {
    pub from_state: String,
    pub to_state: String,
    pub duration_seconds: Real,
    pub elapsed_seconds: Real,
    pub from_time_seconds: Real,
    pub to_time_seconds: Real,
}

fn lock_poison_recovered<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
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

    fn lock_world(&self) -> MutexGuard<'_, World> {
        lock_poison_recovered(&self.inner)
    }

    fn lock_runtime_state(&self) -> MutexGuard<'_, WorldRuntimeState> {
        lock_poison_recovered(&self.runtime_state)
    }

    fn lock_metadata(&self) -> MutexGuard<'_, LevelMetadata> {
        lock_poison_recovered(&self.metadata)
    }

    fn lock_lifecycle(&self) -> MutexGuard<'_, LevelLifecycleState> {
        lock_poison_recovered(&self.lifecycle)
    }

    fn lock_subsystems(&self) -> MutexGuard<'_, Vec<String>> {
        lock_poison_recovered(&self.subsystems)
    }

    pub fn snapshot(&self) -> World {
        self.lock_world().clone()
    }

    pub fn replace(&self, world: World) {
        *self.lock_world() = world;
    }

    pub fn replace_world_and_reset_runtime_state(&self, world: World) {
        self.replace(world);
        *self.lock_runtime_state() = WorldRuntimeState::default();
    }

    pub fn with_world<R>(&self, read: impl FnOnce(&World) -> R) -> R {
        let world = self.lock_world();
        read(&world)
    }

    pub fn with_world_mut<R>(&self, write: impl FnOnce(&mut World) -> R) -> R {
        let mut world = self.lock_world();
        write(&mut world)
    }

    pub fn tick(&self, core: &CoreHandle, advance: RuntimeTimeAdvance) -> Result<(), CoreError> {
        let driver = core.resolve_driver::<WorldDriver>(WORLD_DRIVER_NAME)?;
        driver.tick_level(core, self, advance)
    }

    pub(crate) fn run_runtime_scene_system(
        &self,
        core: &CoreHandle,
        id: &str,
        delta_seconds: Real,
    ) -> Result<bool, CoreError> {
        let Some(mut system) =
            self.with_world_mut(|world| world.schedule_mut().take_runtime_system(id))
        else {
            return Ok(false);
        };

        let result = system.run(RuntimeSceneSystemContext::new(core, self, delta_seconds));
        self.with_world_mut(|world| world.schedule_mut().restore_runtime_system(system));
        result.map(|_| true)
    }

    pub fn animation_pose(&self, entity: EntityId) -> Option<AnimationPoseOutput> {
        self.lock_runtime_state()
            .animation_poses
            .get(&entity)
            .cloned()
    }

    pub(crate) fn animation_poses(&self) -> BTreeMap<EntityId, AnimationPoseOutput> {
        self.lock_runtime_state().animation_poses.clone()
    }

    pub fn animation_playback_times(
        &self,
    ) -> (
        BTreeMap<EntityId, Real>,
        BTreeMap<EntityId, Real>,
        BTreeMap<EntityId, AnimationStateTransitionRuntime>,
    ) {
        let runtime_state = self.lock_runtime_state();
        (
            runtime_state.animation_graph_times.clone(),
            runtime_state.animation_state_machine_times.clone(),
            runtime_state.animation_state_machine_transitions.clone(),
        )
    }

    pub fn record_animation_poses(&self, animation_poses: BTreeMap<EntityId, AnimationPoseOutput>) {
        let mut runtime_state = self.lock_runtime_state();
        runtime_state
            .animation_poses
            .retain(|entity, _| animation_poses.contains_key(entity));
        for (entity, pose) in animation_poses {
            if let Some(existing) = runtime_state.animation_poses.get_mut(&entity) {
                existing.clone_from_reusing_storage(&pose);
            } else {
                runtime_state.animation_poses.insert(entity, pose);
            }
        }
    }

    pub fn record_animation_playback_times(
        &self,
        animation_graph_times: BTreeMap<EntityId, Real>,
        animation_state_machine_times: BTreeMap<EntityId, Real>,
        animation_state_machine_transitions: BTreeMap<EntityId, AnimationStateTransitionRuntime>,
    ) {
        let mut runtime_state = self.lock_runtime_state();
        runtime_state.animation_graph_times = animation_graph_times;
        runtime_state.animation_state_machine_times = animation_state_machine_times;
        runtime_state.animation_state_machine_transitions = animation_state_machine_transitions;
    }

    pub fn script_binding_started(&self, entity: EntityId, binding_key: &str) -> bool {
        self.lock_runtime_state()
            .script_started_bindings
            .contains(&(entity, binding_key.to_string()))
    }

    pub fn mark_script_binding_started(&self, entity: EntityId, binding_key: impl Into<String>) {
        self.lock_runtime_state()
            .script_started_bindings
            .insert((entity, binding_key.into()));
    }

    pub fn metadata(&self) -> LevelMetadata {
        self.lock_metadata().clone()
    }

    pub fn set_metadata(&self, metadata: LevelMetadata) {
        *self.lock_metadata() = metadata;
    }

    pub fn lifecycle(&self) -> LevelLifecycleState {
        self.lock_lifecycle().clone()
    }

    pub fn set_lifecycle(&self, lifecycle: LevelLifecycleState) {
        *self.lock_lifecycle() = lifecycle;
    }

    pub fn register_subsystem(&self, subsystem_name: impl Into<String>) {
        self.lock_subsystems().push(subsystem_name.into());
    }

    pub fn registered_subsystems(&self) -> Vec<String> {
        self.lock_subsystems().clone()
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

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use super::*;

    fn poison_mutex<T>(mutex: &Mutex<T>) {
        let result = catch_unwind(AssertUnwindSafe(|| {
            let _guard = mutex.lock().unwrap();
            panic!("poison level system mutex");
        }));
        assert!(result.is_err());
    }

    #[test]
    fn level_system_accessors_recover_poisoned_state_locks() {
        let level = LevelSystem::new(
            WorldHandle::new(42),
            Arc::new(Mutex::new(World::empty())),
            LevelMetadata::default(),
        );

        poison_mutex(&level.inner);
        let entity = level.with_world_mut(|world| world.spawn_node(crate::scene::NodeKind::Cube));
        assert!(level.snapshot().contains_entity(entity));

        poison_mutex(&level.runtime_state);
        #[cfg(feature = "physics-contracts")]
        assert_eq!(level.last_physics_step_plan(), None);
        level.mark_script_binding_started(entity, "behavior");
        assert!(level.script_binding_started(entity, "behavior"));

        poison_mutex(&level.metadata);
        level.set_metadata(LevelMetadata {
            display_name: Some("Recovered".to_string()),
            ..LevelMetadata::default()
        });
        assert_eq!(level.metadata().display_name.as_deref(), Some("Recovered"));

        poison_mutex(&level.lifecycle);
        level.set_lifecycle(LevelLifecycleState::Unloaded);
        assert_eq!(level.lifecycle(), LevelLifecycleState::Unloaded);

        poison_mutex(&level.subsystems);
        level.register_subsystem("physics");
        assert_eq!(level.registered_subsystems(), vec!["physics".to_string()]);
    }
}

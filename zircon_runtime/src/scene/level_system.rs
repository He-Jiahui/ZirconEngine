//! Runtime level instance wrapping one ECS world plus lifecycle metadata.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use zircon_runtime_interface::world_sync::{
    InvalidationBatch, WatchRegistration, WatchToken, WorldFact,
};

use crate::core::framework::animation::{
    AnimationClipEventBatchAdmission, AnimationClipEventQueueAdmission, AnimationPoseOutput,
};
use crate::core::framework::render::{HighlightSet, ViewportHighlightSet, ViewportHighlightStore};
use crate::core::framework::scene::WorldHandle;
use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle, RuntimeTimeAdvance};
use crate::scene::{
    dynamic_scene::{CompiledSceneSpawn, DynamicScene, DynamicSceneError},
    ecs::RuntimeSceneSystemContext,
    inspection::SubscriptionTable,
    world::World,
    EntityId, EntityRemap, WorldDriver, WORLD_DRIVER_NAME,
};

#[cfg(feature = "animation")]
mod animation_runtime;
mod frame_state;
#[cfg(feature = "physics-contracts")]
#[path = "level_system/physics_runtime_enabled.rs"]
mod physics_runtime;
#[cfg(not(feature = "physics-contracts"))]
#[path = "level_system/physics_runtime_disabled.rs"]
mod physics_runtime;

#[cfg(feature = "animation")]
pub(crate) use frame_state::AnimationPlaybackStateSnapshot;
pub(crate) use frame_state::LevelFrameStateSnapshot;

#[cfg(feature = "animation")]
use frame_state::AnimationRuntimeState;
use frame_state::ScriptRuntimeState;
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
    world_replacement_epoch: Arc<AtomicU64>,
    physics_state: Arc<Mutex<PhysicsRuntimeState>>,
    #[cfg(feature = "animation")]
    animation_state: Arc<Mutex<AnimationRuntimeState>>,
    script_state: Arc<Mutex<ScriptRuntimeState>>,
    frame_state: Arc<Mutex<Arc<LevelFrameStateSnapshot>>>,
    world_subscriptions: Arc<Mutex<SubscriptionTable>>,
    viewport_highlights: Arc<Mutex<ViewportHighlightStore>>,
    metadata: Arc<Mutex<LevelMetadata>>,
    lifecycle: Arc<Mutex<LevelLifecycleState>>,
    subsystems: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone, Debug, PartialEq)]
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
        let world_generation = lock_poison_recovered(&inner).world_generation();
        let world_subscriptions = Arc::new(Mutex::new(SubscriptionTable::default()));
        lock_poison_recovered(&inner)
            .attach_world_sync_subscriptions(Arc::clone(&world_subscriptions));
        Self {
            handle,
            inner,
            world_replacement_epoch: Arc::new(AtomicU64::new(1)),
            physics_state: Arc::new(Mutex::new(PhysicsRuntimeState::default())),
            #[cfg(feature = "animation")]
            animation_state: Arc::new(Mutex::new(AnimationRuntimeState::default())),
            script_state: Arc::new(Mutex::new(ScriptRuntimeState::default())),
            frame_state: Arc::new(Mutex::new(Arc::new(LevelFrameStateSnapshot::new(
                world_generation,
            )))),
            world_subscriptions,
            viewport_highlights: Arc::new(Mutex::new(ViewportHighlightStore::default())),
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

    /// Replaces the editor overlay input retained for one viewport when its
    /// generation is at least as recent as the currently retained value.
    pub fn submit_highlight_set(&self, viewport: u64, generation: u64, set: HighlightSet) -> bool {
        lock_poison_recovered(&self.viewport_highlights).submit(viewport, generation, set)
    }

    pub fn viewport_highlight_set(&self, viewport: u64) -> Option<ViewportHighlightSet> {
        lock_poison_recovered(&self.viewport_highlights)
            .get(viewport)
            .cloned()
    }

    /// Returns the current World generation for work that will publish a sealed frame payload.
    pub fn world_generation(&self) -> u64 {
        self.with_world(World::world_generation)
    }

    /// Captures the identity of the installed World after any replacement reset completes.
    pub fn capture_world_replacement_epoch(&self) -> u64 {
        let _world = self.lock_world();
        self.world_replacement_epoch.load(Ordering::Acquire)
    }

    /// Mutates the World only if it is still the instance identified by `replacement_epoch`.
    pub fn with_world_mut_if_replacement_epoch<R>(
        &self,
        replacement_epoch: u64,
        write: impl FnOnce(&mut World) -> R,
    ) -> Option<R> {
        let mut world = self.lock_world();
        if self.world_replacement_epoch.load(Ordering::Acquire) != replacement_epoch {
            return None;
        }
        Some(write(&mut world))
    }

    /// Registers one runtime-owned world watch for this level session.
    pub fn watch_world(&self, registration: WatchRegistration) -> WatchToken {
        lock_poison_recovered(&self.world_subscriptions).watch(registration)
    }

    /// Revokes a world watch and its pending dirty state.
    pub fn unwatch_world(&self, token: WatchToken) -> bool {
        lock_poison_recovered(&self.world_subscriptions).unwatch(token)
    }

    /// Records one session-level fact that originates outside direct World mutation APIs.
    pub fn record_world_fact(&self, fact: WorldFact) {
        self.lock_world().record_world_fact(fact);
    }

    /// Seals the facts and dirty tokens observed since the previous drain.
    ///
    /// World mutation callbacks lock the world before the subscription table. Holding the same
    /// order here stamps the batch with the generation that covers every included fact.
    pub fn drain_world_invalidations(&self) -> Vec<InvalidationBatch> {
        let world = self.lock_world();
        lock_poison_recovered(&self.world_subscriptions)
            .flush(world.world_generation())
            .into_iter()
            .collect()
    }

    pub(crate) fn lock_world(&self) -> MutexGuard<'_, World> {
        lock_poison_recovered(&self.inner)
    }

    fn lock_physics_state(&self) -> MutexGuard<'_, PhysicsRuntimeState> {
        lock_poison_recovered(&self.physics_state)
    }

    #[cfg(feature = "animation")]
    fn lock_animation_state(&self) -> MutexGuard<'_, AnimationRuntimeState> {
        lock_poison_recovered(&self.animation_state)
    }

    fn lock_script_state(&self) -> MutexGuard<'_, ScriptRuntimeState> {
        lock_poison_recovered(&self.script_state)
    }

    #[cfg(test)]
    fn script_state_generation(&self) -> u64 {
        self.lock_script_state().generation()
    }

    fn lock_frame_state(&self) -> MutexGuard<'_, Arc<LevelFrameStateSnapshot>> {
        lock_poison_recovered(&self.frame_state)
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

    pub(crate) fn dynamic_scene_staging_snapshot(
        &self,
        scene: &DynamicScene,
        limit_bytes: usize,
    ) -> Result<(WorldHandle, u64, World, usize), DynamicSceneError> {
        let mut current = self.lock_world();
        let expected_generation = current.world_generation();
        let (mut snapshot, base_estimated_bytes) =
            current.clone_for_dynamic_scene_staging(limit_bytes)?;
        let estimated_bytes = scene.stage_existing_resources_bounded(
            &current,
            &mut snapshot,
            base_estimated_bytes,
            limit_bytes,
        )?;
        Ok((
            self.world_handle(),
            expected_generation,
            snapshot,
            estimated_bytes,
        ))
    }

    pub(crate) fn dynamic_scene_preflight_snapshot(
        &self,
        scene: &DynamicScene,
        limit_bytes: usize,
    ) -> Result<(WorldHandle, u64, World, CompiledSceneSpawn, usize), DynamicSceneError> {
        let current = self.lock_world();
        let expected_generation = current.world_generation();
        let plan = scene.compile_spawn_into(&current)?;
        let (preflight_world, estimated_bytes) =
            DynamicScene::capture_compiled_spawn_preflight(&current, &plan, limit_bytes)?;
        Ok((
            self.world_handle(),
            expected_generation,
            preflight_world,
            plan,
            estimated_bytes,
        ))
    }

    pub(crate) fn commit_preflighted_dynamic_scene_if_generation(
        &self,
        expected_generation: u64,
        mutation: crate::scene::dynamic_scene::PreflightedSceneMutation,
    ) -> Result<EntityRemap, DynamicSceneError> {
        let mut current = self.lock_world();
        let actual_generation = current.world_generation();
        if actual_generation != expected_generation {
            return Err(DynamicSceneError::TargetWorldChanged {
                expected_generation,
                actual_generation,
            });
        }
        DynamicScene::commit_preflighted_spawn_into(&mut current, mutation)
    }

    pub fn replace(&self, world: World) {
        self.replace_world_and_reset_runtime_state(world);
    }

    pub(crate) fn replace_world_if_generation(
        &self,
        expected_generation: u64,
        world: World,
    ) -> Result<(), u64> {
        let retired = {
            let mut current = self.lock_world();
            let actual_generation = current.world_generation();
            if actual_generation != expected_generation {
                return Err(actual_generation);
            }
            self.world_replacement_epoch.fetch_add(1, Ordering::AcqRel);
            current.clear_all_events();
            let retired = current.commit_staged_scene_state(world);
            current.attach_world_sync_subscriptions(Arc::clone(&self.world_subscriptions));
            self.reset_runtime_state_after_world_replacement(&mut current);
            retired
        };
        drop(retired);
        Ok(())
    }

    pub fn replace_world_and_reset_runtime_state(&self, world: World) {
        let retired = {
            let mut current = self.lock_world();
            self.world_replacement_epoch.fetch_add(1, Ordering::AcqRel);
            let mut world = world;
            world.advance_dynamic_component_generations_after(&current);
            world.advance_scene_binding_generations_after(&current);
            world.advance_world_generation_after(current.world_generation());
            let retired = std::mem::replace(&mut *current, world);
            current.attach_world_sync_subscriptions(Arc::clone(&self.world_subscriptions));
            self.reset_runtime_state_after_world_replacement(&mut current);
            retired
        };
        drop(retired);
    }

    /// Resets World-coupled state while the replacement caller still owns the World lane.
    fn reset_runtime_state_after_world_replacement(&self, current: &mut World) {
        physics_runtime::clear_retained_pose_resources(current);
        let world_generation = current.world_generation();
        let mut frame_state = self.lock_frame_state();
        #[cfg(feature = "animation")]
        {
            Self::publish_animation_frame(
                &mut frame_state,
                world_generation,
                Arc::new(std::collections::BTreeMap::new()),
            );
        }
        #[cfg(not(feature = "animation"))]
        {
            *frame_state = Arc::new(LevelFrameStateSnapshot::new(world_generation));
        }
        drop(frame_state);
        self.lock_physics_state().reset_after_world_replacement();
        #[cfg(feature = "animation")]
        {
            self.lock_animation_state().reset_after_world_replacement();
        }
        self.lock_script_state().reset_after_world_replacement();
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
        self.frame_state_snapshot()
            .animation_poses()
            .get(&entity)
            .cloned()
    }

    pub(crate) fn frame_state_snapshot(&self) -> Arc<LevelFrameStateSnapshot> {
        Arc::clone(&self.lock_frame_state())
    }

    pub fn script_binding_started(&self, entity: EntityId, binding_key: &str) -> bool {
        self.lock_script_state().contains(entity, binding_key)
    }

    pub fn mark_script_binding_started(&self, entity: EntityId, binding_key: impl Into<String>) {
        self.lock_script_state().insert(entity, binding_key.into());
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
    #[cfg(feature = "animation")]
    use std::collections::BTreeMap;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[cfg(feature = "animation")]
    use crate::core::framework::animation::AnimationClipEventSamplingRange;
    use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};
    #[cfg(feature = "animation")]
    use crate::core::resource::ResourceId;

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

        #[cfg(feature = "animation")]
        {
            poison_mutex(&level.animation_state);
            assert_eq!(level.animation_playback_snapshot().generation(), 0);
            level.record_animation_requires_continuous_frame(true);
            assert!(level.animation_requires_continuous_frame());
            let replacement_epoch = level.capture_world_replacement_epoch();
            assert!(level.record_animation_poses(replacement_epoch, BTreeMap::new()));
        }

        poison_mutex(&level.frame_state);
        assert!(level.frame_state_snapshot().animation_poses().is_empty());

        poison_mutex(&level.physics_state);
        #[cfg(feature = "physics-contracts")]
        assert_eq!(level.last_physics_step_plan(), None);

        poison_mutex(&level.script_state);
        level.mark_script_binding_started(entity, "behavior");
        assert!(level.script_binding_started(entity, "behavior"));
        let script_generation = level.script_state_generation();

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

        level.replace_world_and_reset_runtime_state(World::empty());
        assert_eq!(level.script_state_generation(), script_generation + 1);
        assert!(!level.script_binding_started(entity, "behavior"));
    }

    #[test]
    fn world_replacement_advances_generation_past_both_worlds() {
        let mut current = World::empty();
        current.spawn_node(crate::scene::NodeKind::Empty);
        current.spawn_node(crate::scene::NodeKind::Empty);
        let current_generation = current.world_generation();
        let level = LevelSystem::new(
            WorldHandle::new(7),
            Arc::new(Mutex::new(current)),
            LevelMetadata::default(),
        );

        let mut replacement = World::empty();
        replacement.spawn_node(crate::scene::NodeKind::Empty);
        let replacement_generation = replacement.world_generation();
        level.replace(replacement);

        assert_eq!(
            level.with_world(World::world_generation),
            current_generation.max(replacement_generation) + 1
        );
    }

    #[test]
    fn world_replacement_stales_compiled_binding_when_entity_ids_are_reused() {
        let mut current = World::empty();
        let root = current.spawn_node(crate::scene::NodeKind::Empty);
        let hero = current.spawn_node(crate::scene::NodeKind::Mesh);
        current.rename_node(root, "Root").unwrap();
        current.rename_node(hero, "Hero").unwrap();
        current.set_parent_checked(hero, Some(root)).unwrap();
        let writer = current
            .compile_scene_property_writer(
                &EntityPath::parse("Root/Hero").unwrap(),
                &ComponentPropertyPath::parse("Transform.translation").unwrap(),
            )
            .unwrap()
            .unwrap();
        let level = LevelSystem::new(
            WorldHandle::new(8),
            Arc::new(Mutex::new(current)),
            LevelMetadata::default(),
        );

        let mut replacement = World::empty();
        let replacement_root = replacement.spawn_node(crate::scene::NodeKind::Empty);
        let replacement_hero = replacement.spawn_node(crate::scene::NodeKind::Mesh);
        assert_eq!(root, replacement_root);
        assert_eq!(hero, replacement_hero);
        replacement.rename_node(replacement_root, "Root").unwrap();
        replacement.rename_node(replacement_hero, "Hero").unwrap();
        replacement
            .set_parent_checked(replacement_hero, Some(replacement_root))
            .unwrap();

        level.replace(replacement);

        assert!(level.with_world(|world| !writer.is_current_for(world)));
    }

    #[cfg(feature = "animation")]
    #[test]
    fn animation_clip_event_backlog_is_reset_with_the_replaced_world() {
        let level = LevelSystem::new(
            WorldHandle::new(9),
            Arc::new(Mutex::new(World::empty())),
            LevelMetadata::default(),
        );
        let replacement_epoch = level.capture_world_replacement_epoch();
        assert_eq!(
            level.enqueue_animation_clip_event_range_batches(
                replacement_epoch,
                vec![vec![AnimationClipEventSamplingRange {
                    entity: 17,
                    clip_id: ResourceId::from_stable_label("animation.pending-event"),
                    from_time_seconds: 0.0,
                    to_time_seconds: 120.0,
                    looping: true,
                }]],
            ),
            AnimationClipEventQueueAdmission::Current {
                batch_admissions: vec![AnimationClipEventBatchAdmission::Admitted],
                admitted_range_count: 1,
                deferred_range_count: 0,
                rejected_range_count: 0,
            },
        );

        assert_eq!(
            level.animation_clip_event_backlog_len(replacement_epoch),
            Some(1)
        );
        level.replace_world_and_reset_runtime_state(World::empty());
        let current_epoch = level.capture_world_replacement_epoch();
        assert_eq!(
            level.animation_clip_event_backlog_len(current_epoch),
            Some(0)
        );
        assert_eq!(level.animation_clip_event_drain_metrics().0, 0);
    }

    #[cfg(not(feature = "animation"))]
    #[test]
    fn level_system_constructs_and_replaces_world_without_animation() {
        let level = LevelSystem::new(
            WorldHandle::new(10),
            Arc::new(Mutex::new(World::empty())),
            LevelMetadata::default(),
        );

        let before = level.world_generation();
        level.replace_world_and_reset_runtime_state(World::empty());

        assert!(level.world_generation() > before);
    }
}

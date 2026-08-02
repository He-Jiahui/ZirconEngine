//! Runtime level instance wrapping one ECS world plus lifecycle metadata.

use std::sync::{Arc, Mutex, MutexGuard};

#[cfg(feature = "animation")]
use std::collections::BTreeMap;

#[cfg(feature = "animation")]
use crate::animation::{
    AnimationClipEvent, AnimationClipEventSamplingLimits, sample_clip_events_budgeted,
};
#[cfg(feature = "animation")]
use crate::asset::{AssetId, ProjectAssetManager};
use crate::core::framework::animation::AnimationPoseOutput;
use crate::core::framework::scene::WorldHandle;
use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle, RuntimeTimeAdvance};
use crate::scene::{
    EntityId, EntityRemap, WORLD_DRIVER_NAME, WorldDriver,
    dynamic_scene::{CompiledSceneSpawn, DynamicScene, DynamicSceneError},
    ecs::RuntimeSceneSystemContext,
    world::World,
};

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

use frame_state::ScriptRuntimeState;
#[cfg(feature = "animation")]
use frame_state::{AnimationClipEventDrainMetrics, AnimationRuntimeState};
use physics_runtime::PhysicsRuntimeState;

#[cfg(feature = "animation")]
const ANIMATION_CLIP_EVENT_MAX_DRAIN_SAMPLES: usize = 32;

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
    physics_state: Arc<Mutex<PhysicsRuntimeState>>,
    #[cfg(feature = "animation")]
    animation_state: Arc<Mutex<AnimationRuntimeState>>,
    script_state: Arc<Mutex<ScriptRuntimeState>>,
    frame_state: Arc<Mutex<Arc<LevelFrameStateSnapshot>>>,
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
        Self {
            handle,
            inner,
            physics_state: Arc::new(Mutex::new(PhysicsRuntimeState::default())),
            #[cfg(feature = "animation")]
            animation_state: Arc::new(Mutex::new(AnimationRuntimeState::default())),
            script_state: Arc::new(Mutex::new(ScriptRuntimeState::default())),
            frame_state: Arc::new(Mutex::new(Arc::new(LevelFrameStateSnapshot::new(
                world_generation,
            )))),
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

    /// Returns the current World generation for work that will publish a sealed frame payload.
    pub fn world_generation(&self) -> u64 {
        self.with_world(World::world_generation)
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
            scene.capture_compiled_spawn_preflight(&current, &plan, limit_bytes)?;
        Ok((
            self.world_handle(),
            expected_generation,
            preflight_world,
            plan,
            estimated_bytes,
        ))
    }

    pub(crate) fn apply_preflighted_dynamic_scene_if_generation(
        &self,
        expected_generation: u64,
        scene: &DynamicScene,
        plan: CompiledSceneSpawn,
    ) -> Result<EntityRemap, DynamicSceneError> {
        let mut current = self.lock_world();
        let actual_generation = current.world_generation();
        if actual_generation != expected_generation {
            return Err(DynamicSceneError::TargetWorldChanged {
                expected_generation,
                actual_generation,
            });
        }
        scene.apply_preflighted_compiled_spawn_into(&mut current, plan)
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
            current.commit_staged_scene_state(world)
        };
        drop(retired);
        Ok(())
    }

    pub fn replace_world_and_reset_runtime_state(&self, world: World) {
        // Frame publication and replacement use the same World -> frame-state order so a
        // producer from the retired world cannot publish after this reset completes.
        let mut current = self.lock_world();
        let mut world = world;
        world.advance_dynamic_component_generations_after(&current);
        world.advance_scene_binding_generations_after(&current);
        world.advance_world_generation_after(current.world_generation());
        *current = world;
        let world_generation = current.world_generation();
        let mut frame_state = self.lock_frame_state();
        #[cfg(feature = "animation")]
        {
            Self::publish_animation_frame(
                &mut frame_state,
                world_generation,
                Arc::new(BTreeMap::new()),
            );
        }
        #[cfg(not(feature = "animation"))]
        {
            *frame_state = Arc::new(LevelFrameStateSnapshot::new(world_generation));
        }
        drop(frame_state);
        drop(current);

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

    #[cfg(feature = "animation")]
    pub(crate) fn animation_requires_continuous_frame(&self) -> bool {
        let state = self.lock_animation_state();
        state.animation_requires_continuous_frame
            || state.animation_event_backlog_requires_continuous_frame
    }

    #[cfg(feature = "animation")]
    pub fn record_animation_requires_continuous_frame(&self, requires_continuous_frame: bool) {
        self.lock_animation_state()
            .animation_requires_continuous_frame = requires_continuous_frame;
    }

    #[cfg(feature = "animation")]
    pub fn record_animation_event_backlog_continuity(&self, requires_continuous_frame: bool) {
        self.lock_animation_state()
            .animation_event_backlog_requires_continuous_frame = requires_continuous_frame;
    }

    #[cfg(feature = "animation")]
    pub(crate) fn animation_playback_snapshot(&self) -> Arc<AnimationPlaybackStateSnapshot> {
        Arc::clone(&self.lock_animation_state().playback_state)
    }

    #[cfg(feature = "animation")]
    pub fn enqueue_animation_clip_event_range(
        &self,
        entity: EntityId,
        clip_id: AssetId,
        from_time_seconds: Real,
        to_time_seconds: Real,
        looping: bool,
    ) {
        self.lock_animation_state().enqueue_clip_event_sample(
            entity,
            clip_id,
            from_time_seconds,
            to_time_seconds,
            looping,
        );
    }

    #[cfg(feature = "animation")]
    pub fn drain_animation_clip_events(
        &self,
        asset_manager: &ProjectAssetManager,
    ) -> Vec<AnimationClipEvent> {
        let limits = AnimationClipEventSamplingLimits::default();
        let mut events = Vec::new();
        let mut emitted_event_bytes: usize = 0;
        let mut metrics = AnimationClipEventDrainMetrics::default();

        for _ in 0..ANIMATION_CLIP_EVENT_MAX_DRAIN_SAMPLES {
            let Some(mut pending) = self.lock_animation_state().take_clip_event_sample() else {
                break;
            };
            let Some(remaining_event_bytes) =
                limits.max_event_bytes.checked_sub(emitted_event_bytes)
            else {
                pending.age_frames = pending.age_frames.saturating_add(1);
                self.lock_animation_state()
                    .requeue_clip_event_sample_front(pending);
                metrics.budget_exhausted = true;
                break;
            };
            if remaining_event_bytes == 0 {
                pending.age_frames = pending.age_frames.saturating_add(1);
                self.lock_animation_state()
                    .requeue_clip_event_sample_front(pending);
                metrics.budget_exhausted = true;
                break;
            }
            let remaining_events = limits.max_events.saturating_sub(events.len());
            if remaining_events == 0 {
                pending.age_frames = pending.age_frames.saturating_add(1);
                self.lock_animation_state()
                    .requeue_clip_event_sample_front(pending);
                metrics.budget_exhausted = true;
                break;
            }
            let Ok(clip) = asset_manager.load_animation_clip_asset(pending.clip_id) else {
                pending.age_frames = pending.age_frames.saturating_add(1);
                self.lock_animation_state()
                    .requeue_clip_event_sample_back(pending);
                metrics.unavailable_asset_count = metrics.unavailable_asset_count.saturating_add(1);
                continue;
            };

            let batch = sample_clip_events_budgeted(
                &clip,
                pending.entity,
                pending.from_time_seconds,
                pending.to_time_seconds,
                pending.looping,
                Some(pending.cursor.clone()),
                AnimationClipEventSamplingLimits {
                    max_events: remaining_events,
                    max_event_bytes: remaining_event_bytes,
                    max_playback_span_seconds: limits.max_playback_span_seconds,
                },
            );
            emitted_event_bytes = emitted_event_bytes.saturating_add(batch.emitted_event_bytes);
            metrics.oversized_event_count = metrics
                .oversized_event_count
                .saturating_add(batch.oversized_event_count);
            metrics.budget_exhausted |= batch.budget_exhausted;
            events.extend(batch.events);
            if let Some(cursor) = batch.next_cursor {
                pending.cursor = cursor;
                pending.age_frames = pending.age_frames.saturating_add(1);
                self.lock_animation_state()
                    .requeue_clip_event_sample_front(pending);
                break;
            }
        }

        let (deferred_range_count, oldest_pending_age_frames) =
            self.lock_animation_state().clip_event_backlog();
        metrics.deferred_range_count = deferred_range_count;
        metrics.oldest_pending_age_frames = oldest_pending_age_frames;
        self.lock_animation_state().record_clip_event_drain(metrics);
        events
    }

    #[cfg(feature = "animation")]
    pub fn animation_clip_event_backlog_len(&self) -> usize {
        self.lock_animation_state().clip_event_backlog().0
    }

    #[cfg(feature = "animation")]
    pub(crate) fn animation_clip_event_drain_metrics(&self) -> (usize, u64, bool, usize, usize) {
        let metrics = self.lock_animation_state().last_clip_event_drain();
        (
            metrics.deferred_range_count,
            metrics.oldest_pending_age_frames,
            metrics.budget_exhausted,
            metrics.oversized_event_count,
            metrics.unavailable_asset_count,
        )
    }

    #[cfg(feature = "animation")]
    pub fn record_animation_poses(
        &self,
        world_generation: u64,
        animation_poses: BTreeMap<EntityId, crate::core::framework::animation::AnimationPoseOutput>,
    ) -> bool {
        self.record_animation_pose_snapshot(world_generation, Arc::new(animation_poses))
    }

    /// Publishes an immutable animation pose snapshot without cloning a
    /// plugin-owned paused-pose cache on frames that produced no new samples.
    ///
    /// The caller must capture `world_generation` before producing the payload. A false return
    /// means replacement retired that generation, so the payload must not be applied to the new
    /// World.
    #[cfg(feature = "animation")]
    pub fn record_animation_pose_snapshot(
        &self,
        world_generation: u64,
        animation_poses: Arc<
            BTreeMap<EntityId, crate::core::framework::animation::AnimationPoseOutput>,
        >,
    ) -> bool {
        let published = self.frame_state_snapshot();
        let published_matches_payload =
            published.animation_poses().as_ref() == animation_poses.as_ref();
        let world = self.lock_world();
        if world_generation != world.world_generation() {
            return false;
        }
        let mut current = self.lock_frame_state();
        if published_matches_payload
            && published.world_generation() == world_generation
            && Arc::ptr_eq(&published, &current)
        {
            return true;
        }
        if current.world_generation() == world_generation
            && Arc::ptr_eq(current.animation_poses(), &animation_poses)
        {
            return true;
        }

        Self::publish_animation_frame(&mut current, world_generation, animation_poses);
        true
    }

    #[cfg(feature = "animation")]
    pub fn record_animation_playback_times(
        &self,
        animation_graph_times: BTreeMap<EntityId, Real>,
        animation_state_machine_times: BTreeMap<EntityId, Real>,
        animation_state_machine_transitions: BTreeMap<EntityId, AnimationStateTransitionRuntime>,
    ) {
        let mut animation_state = self.lock_animation_state();
        let published = &animation_state.playback_state;
        if published.animation_graph_times().as_ref() == &animation_graph_times
            && published.animation_state_machine_times().as_ref() == &animation_state_machine_times
            && published.animation_state_machine_transitions().as_ref()
                == &animation_state_machine_transitions
        {
            return;
        }

        animation_state.playback_state = Arc::new(published.with_values(
            animation_graph_times,
            animation_state_machine_times,
            animation_state_machine_transitions,
        ));
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

    #[cfg(feature = "animation")]
    fn publish_animation_frame(
        frame_state: &mut Arc<LevelFrameStateSnapshot>,
        world_generation: u64,
        animation_poses: Arc<
            BTreeMap<EntityId, crate::core::framework::animation::AnimationPoseOutput>,
        >,
    ) {
        *frame_state =
            Arc::new(frame_state.with_animation_poses(world_generation, animation_poses));
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
    use std::panic::{AssertUnwindSafe, catch_unwind};

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
            assert!(level.record_animation_poses(level.world_generation(), BTreeMap::new()));
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
        level.enqueue_animation_clip_event_range(
            17,
            ResourceId::from_stable_label("animation.pending-event"),
            0.0,
            120.0,
            true,
        );

        assert_eq!(level.animation_clip_event_backlog_len(), 1);
        level.replace_world_and_reset_runtime_state(World::empty());
        assert_eq!(level.animation_clip_event_backlog_len(), 0);
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

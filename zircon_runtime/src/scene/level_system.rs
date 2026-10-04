//! Runtime level instance wrapping one ECS world plus lifecycle metadata.

use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use zircon_runtime_interface::world_sync::{
    InvalidationBatch, WatchRegistration, WatchToken, WorldFact, WorldQuery, WorldQueryResult,
};

use crate::core::framework::animation::{
    AnimationClipEventBatchAdmission, AnimationClipEventQueueAdmission, AnimationPoseOutput,
};
use crate::core::framework::render::{HighlightSet, ViewportHighlightSet, ViewportHighlightStore};
use crate::core::framework::scene::WorldHandle;
use crate::core::framework::time::{TimePolicy, TimePolicyError, TimePolicyTransaction};
use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle, FrameTimeSnapshot};
use crate::scene::world_publication::{WorldPublicationOwner, WorldPublicationSource};
use crate::scene::world_time::{
    FixedInterpolationContext, WorldFixedStep, WorldFixedStepError, WorldTimeAdvanceError,
    WorldTimeControlError, WorldTimeController, WorldTimeSnapshot,
};
use crate::scene::{
    dynamic_scene::{CompiledSceneSpawn, DynamicScene, DynamicSceneError},
    ecs::{RuntimeSceneSystemContext, SystemTickContext},
    inspection::SubscriptionTable,
    world::World,
    EntityId, EntityRemap, LevelTickError, WorldDriver, WorldTimePolicyReceipt, WorldTimeState,
    WORLD_DRIVER_NAME,
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
    publication_owner: Arc<WorldPublicationOwner>,
    physics_state: Arc<Mutex<PhysicsRuntimeState>>,
    #[cfg(feature = "animation")]
    animation_state: Arc<Mutex<AnimationRuntimeState>>,
    script_state: Arc<Mutex<ScriptRuntimeState>>,
    frame_state: Arc<Mutex<Arc<LevelFrameStateSnapshot>>>,
    world_subscriptions: Arc<Mutex<SubscriptionTable>>,
    viewport_highlights: Arc<Mutex<ViewportHighlightStore>>,
    world_time: Arc<Mutex<WorldTimeController>>,
    metadata: Arc<Mutex<LevelMetadata>>,
    lifecycle: Arc<Mutex<LevelLifecycleState>>,
    subsystems: Arc<Mutex<Arc<[String]>>>,
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
        let world_replacement_epoch = Arc::new(AtomicU64::new(1));
        let publication_owner = Arc::new(WorldPublicationOwner {
            world: Arc::clone(&inner),
            replacement_epoch: Arc::clone(&world_replacement_epoch),
            lifecycle_epoch: AtomicU64::new(1),
        });
        Self {
            handle,
            inner,
            world_replacement_epoch,
            publication_owner,
            physics_state: Arc::new(Mutex::new(PhysicsRuntimeState::default())),
            #[cfg(feature = "animation")]
            animation_state: Arc::new(Mutex::new(AnimationRuntimeState::default())),
            script_state: Arc::new(Mutex::new(ScriptRuntimeState::default())),
            frame_state: Arc::new(Mutex::new(Arc::new(LevelFrameStateSnapshot::new(
                world_generation,
            )))),
            world_subscriptions,
            viewport_highlights: Arc::new(Mutex::new(ViewportHighlightStore::default())),
            world_time: Arc::new(Mutex::new(WorldTimeController::default())),
            metadata: Arc::new(Mutex::new(metadata)),
            lifecycle: Arc::new(Mutex::new(LevelLifecycleState::Loaded)),
            subsystems: Arc::new(Mutex::new(Arc::default())),
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

    /// Returns an immutable copy of this Level's virtual and fixed clocks.
    pub fn world_time(&self) -> WorldTimeState {
        lock_poison_recovered(&self.world_time).state()
    }

    /// Returns interpolation evidence built only from committed fixed states.
    pub fn fixed_interpolation_context(&self) -> FixedInterpolationContext {
        lock_poison_recovered(&self.world_time).fixed_interpolation_context()
    }

    pub fn time_policy(&self) -> TimePolicy {
        lock_poison_recovered(&self.world_time).time_policy()
    }

    pub fn apply_time_policy(
        &self,
        transaction: TimePolicyTransaction,
    ) -> Result<WorldTimePolicyReceipt, TimePolicyError> {
        lock_poison_recovered(&self.world_time).apply_time_policy(transaction)
    }

    /// Installs the policy selected before this Level is published to callers.
    pub(crate) fn initialize_time_policy(&self, policy: TimePolicy) -> Result<(), TimePolicyError> {
        *lock_poison_recovered(&self.world_time) = WorldTimeController::new(policy)?;
        Ok(())
    }

    pub fn pause_virtual_time(&self) {
        lock_poison_recovered(&self.world_time).pause_virtual_time();
    }

    pub fn unpause_virtual_time(&self) {
        lock_poison_recovered(&self.world_time).unpause_virtual_time();
    }

    pub fn request_single_step(&self) -> Result<(), WorldTimeControlError> {
        lock_poison_recovered(&self.world_time).request_single_step()
    }

    pub(crate) fn advance_world_time(
        &self,
        outer: FrameTimeSnapshot,
    ) -> Result<WorldTimeSnapshot, WorldTimeAdvanceError> {
        lock_poison_recovered(&self.world_time).advance(outer)
    }

    pub(crate) fn begin_fixed_step(
        &self,
        world_generation: u64,
    ) -> Result<WorldFixedStep, WorldFixedStepError> {
        lock_poison_recovered(&self.world_time).begin_fixed_step(world_generation)
    }

    pub(crate) fn commit_fixed_step(
        &self,
        expected_world_generation: u64,
        step: &WorldFixedStep,
    ) -> Result<(), WorldFixedStepError> {
        let world = self.lock_world();
        let actual_world_generation = world.world_generation();
        if actual_world_generation != expected_world_generation {
            return Err(WorldFixedStepError::WorldGenerationChanged {
                expected: expected_world_generation,
                actual: actual_world_generation,
            });
        }
        lock_poison_recovered(&self.world_time).commit_fixed_step(step)
    }

    pub(crate) fn abort_fixed_step(&self, step: WorldFixedStep) -> Result<(), WorldFixedStepError> {
        lock_poison_recovered(&self.world_time).abort_fixed_step(step)
    }

    /// Captures the identity of the installed World after any replacement reset completes.
    pub fn capture_world_replacement_epoch(&self) -> u64 {
        let _world = self.lock_world();
        self.world_replacement_epoch.load(Ordering::Acquire)
    }

    fn advance_world_replacement_epoch(&self) -> u64 {
        let previous = self
            .world_replacement_epoch
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |epoch| {
                epoch.checked_add(1)
            })
            .expect("world replacement epoch exhausted");
        previous
            .checked_add(1)
            .expect("world replacement epoch was checked before publication")
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

    fn lock_subsystems(&self) -> MutexGuard<'_, Arc<[String]>> {
        lock_poison_recovered(&self.subsystems)
    }

    pub fn snapshot(&self) -> World {
        self.lock_world().clone()
    }

    /// Captures one immutable World clone and its typed live publication source under the same
    /// authoritative World lock. Expensive preparation can proceed without retaining that lock;
    /// terminal publication must use the returned source.
    pub fn capture(&self) -> WorldPublicationSource {
        let world = self.lock_world();
        let _lifecycle = self.lock_lifecycle();
        let generation = world.world_generation();
        let replacement_epoch = self.world_replacement_epoch.load(Ordering::Acquire);
        WorldPublicationSource::from_capture(
            Arc::downgrade(&self.publication_owner),
            Arc::downgrade(&self.lifecycle),
            world.clone(),
            generation,
            replacement_epoch,
            self.publication_owner
                .lifecycle_epoch
                .load(Ordering::Acquire),
        )
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
            let replacement_epoch = self.advance_world_replacement_epoch();
            current.clear_all_events();
            let retired = current.commit_staged_scene_state(world);
            current.attach_world_sync_subscriptions(Arc::clone(&self.world_subscriptions));
            self.reset_runtime_state_after_world_replacement(&mut current);
            current.record_world_fact(WorldFact::WorldReplaced { replacement_epoch });
            retired
        };
        drop(retired);
        Ok(())
    }

    pub fn replace_world_and_reset_runtime_state(&self, world: World) {
        let retired = {
            let mut current = self.lock_world();
            let replacement_epoch = self.advance_world_replacement_epoch();
            let mut world = world;
            world.advance_dynamic_component_generations_after(&current);
            world.advance_scene_binding_generations_after(&current);
            world.advance_world_generation_after(current.world_generation());
            let retired = std::mem::replace(&mut *current, world);
            current.attach_world_sync_subscriptions(Arc::clone(&self.world_subscriptions));
            self.reset_runtime_state_after_world_replacement(&mut current);
            current.record_world_fact(WorldFact::WorldReplaced { replacement_epoch });
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
        lock_poison_recovered(&self.world_time).reset_after_world_replacement();
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

    /// Runs one transport-neutral query against a single World replacement identity.
    pub fn query_world(&self, query: &WorldQuery) -> WorldQueryResult {
        self.with_world_and_replacement_epoch(|world, replacement_epoch| {
            world.query_world_at_replacement_epoch(query, replacement_epoch)
        })
    }

    pub(crate) fn with_world_and_replacement_epoch<R>(
        &self,
        read: impl FnOnce(&World, u64) -> R,
    ) -> R {
        let world = self.lock_world();
        let replacement_epoch = self.world_replacement_epoch.load(Ordering::Acquire);
        read(&world, replacement_epoch)
    }

    pub fn with_world_mut<R>(&self, write: impl FnOnce(&mut World) -> R) -> R {
        let mut world = self.lock_world();
        write(&mut world)
    }

    pub(crate) fn with_world_mut_and_replacement_epoch<R>(
        &self,
        write: impl FnOnce(&mut World, u64) -> R,
    ) -> R {
        let mut world = self.lock_world();
        let replacement_epoch = self.world_replacement_epoch.load(Ordering::Acquire);
        write(&mut world, replacement_epoch)
    }

    /// Advances this level through its registered world driver.
    ///
    /// The plugin SDK's transport-neutral test runtime uses the same public
    /// tick boundary as the host, so shipping builds do not need a
    /// test-only/private escape hatch.
    pub fn tick(
        &self,
        core: &CoreHandle,
        snapshot: FrameTimeSnapshot,
    ) -> Result<(), LevelTickError> {
        let driver = core.resolve_driver::<WorldDriver>(WORLD_DRIVER_NAME)?;
        driver.tick_level(core, self, snapshot)
    }

    pub(crate) fn run_runtime_scene_system(
        &self,
        core: &CoreHandle,
        id: &str,
        tick: SystemTickContext,
    ) -> Result<bool, CoreError> {
        let (system, replacement_epoch) =
            self.with_world_mut_and_replacement_epoch(|world, replacement_epoch| {
                (
                    world.schedule_mut().take_runtime_system(id),
                    replacement_epoch,
                )
            });
        let Some(mut system) = system else {
            return Ok(false);
        };

        let result = catch_unwind(AssertUnwindSafe(|| {
            system.run(RuntimeSceneSystemContext::new(core, self, tick))
        }));
        let _ = self.with_world_mut_if_replacement_epoch(replacement_epoch, move |world| {
            world.schedule_mut().restore_runtime_system(system);
        });
        match result {
            Ok(result) => result.map(|_| true),
            Err(payload) => resume_unwind(payload),
        }
    }

    pub fn animation_pose(&self, entity: EntityId) -> Option<AnimationPoseOutput> {
        self.frame_state_snapshot()
            .animation_poses()
            .get(&entity)
            .map(|pose| pose.as_ref().clone())
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
        let mut current = self.lock_lifecycle();
        if *current != lifecycle {
            self.publication_owner
                .lifecycle_epoch
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |epoch| {
                    epoch.checked_add(1)
                })
                .expect("level publication lifecycle epoch exhausted");
            *current = lifecycle;
        }
    }

    pub fn register_subsystem(&self, subsystem_name: impl Into<String>) {
        let mut registry = self.lock_subsystems();
        let mut names = registry.as_ref().to_vec();
        names.push(subsystem_name.into());
        *registry = names.into();
    }

    pub fn registered_subsystems(&self) -> Arc<[String]> {
        Arc::clone(&self.lock_subsystems())
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
#[path = "tests/level_system.rs"]
mod tests;

#[cfg(test)]
#[path = "level_system/tests/subsystem_snapshot_tests.rs"]
mod subsystem_snapshot_tests;

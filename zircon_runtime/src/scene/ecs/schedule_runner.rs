use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::time::Instant;

use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle, JobScheduler};
use crate::scene::ecs::{
    BoxedSceneSystem, DeferredSystemKey, InternalSceneSystem, NativeSystemCallbackTiming,
    SceneSystemDescriptor, ScheduleConflictGraph, ScheduledSceneStep, ScheduledSceneStepRef,
    SystemStage,
};
use crate::scene::LevelSystem;

pub(crate) struct SceneScheduleRunner;

struct WorkerDispatch<'a> {
    id: &'a str,
    key: DeferredSystemKey,
}

impl SceneScheduleRunner {
    pub(crate) fn run_stage(
        core: &CoreHandle,
        level: &LevelSystem,
        stage: SystemStage,
        delta_seconds: Real,
        internal_systems: &[SceneSystemDescriptor],
        native_steps: &[ScheduledSceneStep],
        native_conflicts: &ScheduleConflictGraph,
    ) -> Result<(), CoreError> {
        crate::profile_scope!("runtime", "frame", schedule_stage_profile_name(stage),);

        level.with_world_mut(|world| world.set_scene_system_flush_deferred(true));
        level.with_world_mut(|world| {
            world.record_native_system_conflicts(native_conflicts.edges().len())
        });

        let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), CoreError> {
            let mut worker_batch = Vec::new();
            for step in
                ScheduledSceneStep::iter_sorted_for_stage(stage, internal_systems, native_steps)
            {
                match step {
                    ScheduledSceneStepRef::Internal(system) => {
                        flush_worker_batch(core.scheduler(), level, &mut worker_batch)?;
                        level.with_world_mut(|world| {
                            world.run_internal_scene_system(system.system())
                        });
                        if !matches!(
                            system.system(),
                            InternalSceneSystem::ApplyDeferred | InternalSceneSystem::UpdateEvents
                        ) {
                            level.with_world_mut(|world| world.apply_deferred());
                        }
                    }
                    ScheduledSceneStepRef::Native {
                        id,
                        stage: step_stage,
                        order,
                        worker_safe,
                        conservative_world_writer,
                    } => {
                        if worker_safe {
                            if worker_batch
                                .iter()
                                .any(|other| native_conflicts.systems_conflict(other.id, id))
                            {
                                flush_worker_batch(core.scheduler(), level, &mut worker_batch)?;
                            }
                            worker_batch.push(WorkerDispatch {
                                id,
                                key: DeferredSystemKey::compiled(step_stage.rank(), order, id),
                            });
                        } else {
                            flush_worker_batch(core.scheduler(), level, &mut worker_batch)?;
                            level.with_world_mut(|world| {
                                let started_at = Instant::now();
                                let result = catch_unwind(AssertUnwindSafe(|| {
                                    world.run_native_scene_system(id)
                                }));
                                world.record_native_system_main_callback(
                                    started_at.elapsed(),
                                    conservative_world_writer,
                                );
                                if let Err(payload) = result {
                                    resume_unwind(payload);
                                }
                            });
                        }
                    }
                    ScheduledSceneStepRef::Runtime { id, .. } => {
                        flush_worker_batch(core.scheduler(), level, &mut worker_batch)?;
                        level.run_runtime_scene_system(core, id, delta_seconds)?;
                        level.with_world_mut(|world| world.apply_deferred());
                    }
                    ScheduledSceneStepRef::ApplyDeferred { .. } => {
                        flush_worker_batch(core.scheduler(), level, &mut worker_batch)?;
                        level.with_world_mut(|world| world.apply_deferred());
                    }
                }
            }
            flush_worker_batch(core.scheduler(), level, &mut worker_batch)?;

            Ok(())
        }));
        let stage_succeeded = matches!(&result, Ok(Ok(())));
        level.with_world_mut(|world| {
            world.set_scene_system_flush_deferred(false);
            if stage_succeeded {
                world.flush_pending_scene_systems_for_stage(stage);
            }
        });
        match result {
            Ok(result) => result,
            Err(payload) => resume_unwind(payload),
        }
    }
}

fn flush_worker_batch(
    scheduler: &JobScheduler,
    level: &LevelSystem,
    dispatches: &mut Vec<WorkerDispatch<'_>>,
) -> Result<(), CoreError> {
    if dispatches.is_empty() {
        return Ok(());
    }
    let ready_at = Instant::now();
    let system_ids = dispatches
        .iter()
        .map(|dispatch| dispatch.id)
        .collect::<Vec<_>>();
    let mut systems = level
        .with_world_mut(|world| world.take_worldless_native_scene_systems(&system_ids))
        .expect("compiled worker-safe scene systems must remain registered for the stage");
    for (system, dispatch) in systems.iter_mut().zip(dispatches.iter()) {
        if let Some(buffer) = system.worker_command_buffer_mut() {
            buffer.bind_compiled_key(dispatch.key.clone());
        }
    }
    let mut timings = vec![NativeSystemCallbackTiming::default(); systems.len()];
    let mut temporary_control_buffer_count = 2;
    let mut temporary_control_buffer_bytes = systems
        .capacity()
        .saturating_mul(std::mem::size_of::<BoxedSceneSystem>())
        .saturating_add(
            timings
                .capacity()
                .saturating_mul(std::mem::size_of::<NativeSystemCallbackTiming>()),
        );
    let batch_started_at = Instant::now();
    let result = catch_unwind(AssertUnwindSafe(|| {
        run_worldless_systems(scheduler, &mut systems, &mut timings, ready_at);
        let mut command_buffers = systems
            .iter_mut()
            .filter_map(|system| system.worker_command_buffer_mut())
            .collect::<Vec<_>>();
        if command_buffers.capacity() > 0 {
            temporary_control_buffer_count += 1;
            temporary_control_buffer_bytes = temporary_control_buffer_bytes.saturating_add(
                command_buffers
                    .capacity()
                    .saturating_mul(std::mem::size_of::<
                        &mut crate::scene::ecs::WorkerCommandBuffer,
                    >()),
            );
        }
        let has_worker_commands = !command_buffers.is_empty();
        level.with_world_mut(|world| {
            world.merge_worker_command_buffers(&mut command_buffers)?;
            if has_worker_commands {
                let apply_result = catch_unwind(AssertUnwindSafe(|| world.apply_deferred()));
                world.reclaim_worker_command_buffers(&mut command_buffers);
                if let Err(payload) = apply_result {
                    resume_unwind(payload);
                }
            }
            Ok(())
        })
    }));
    // A worker callback may have completed before a sibling panics, or the
    // merge itself may reject the whole batch. Only a fully delivered batch
    // may return its lanes to the registry with their payloads intact.
    if !matches!(&result, Ok(Ok(()))) {
        for system in &mut systems {
            if let Some(buffer) = system.worker_command_buffer_mut() {
                buffer.discard_pending();
            }
        }
    }
    let batch_elapsed = batch_started_at.elapsed();
    level.with_world_mut(|world| {
        world.restore_worldless_native_scene_systems(systems);
        world.record_native_system_worker_batch(
            &timings,
            batch_elapsed,
            scheduler.parallelism(),
            temporary_control_buffer_count,
            temporary_control_buffer_bytes,
        );
    });
    dispatches.clear();
    let command_buffer_result: Result<(), crate::scene::ecs::WorkerCommandBufferMergeError> =
        match result {
            Ok(result) => result,
            Err(payload) => resume_unwind(payload),
        };
    command_buffer_result.map_err(|error| {
        CoreError::Initialization(
            "SceneScheduleRunner worker command buffer merge".to_string(),
            error.to_string(),
        )
    })
}

fn run_worldless_systems(
    scheduler: &JobScheduler,
    systems: &mut [BoxedSceneSystem],
    timings: &mut [NativeSystemCallbackTiming],
    ready_at: Instant,
) {
    debug_assert_eq!(systems.len(), timings.len());
    match (systems, timings) {
        ([], []) => {}
        ([system], [timing]) => run_worldless_system(system, timing, ready_at),
        (systems, timings) => {
            let midpoint = systems.len() / 2;
            let (left, right) = systems.split_at_mut(midpoint);
            let (left_timings, right_timings) = timings.split_at_mut(midpoint);
            scheduler.join(
                || run_worldless_systems(scheduler, left, left_timings, ready_at),
                || run_worldless_systems(scheduler, right, right_timings, ready_at),
            );
        }
    }
}

fn run_worldless_system(
    system: &mut BoxedSceneSystem,
    timing: &mut NativeSystemCallbackTiming,
    ready_at: Instant,
) {
    let started_at = Instant::now();
    timing.ready_delay = started_at.saturating_duration_since(ready_at);
    let result = catch_unwind(AssertUnwindSafe(|| system.run_without_world()));
    timing.callback = started_at.elapsed();
    if let Err(payload) = result {
        if let Some(buffer) = system.worker_command_buffer_mut() {
            buffer.discard_pending();
        }
        resume_unwind(payload);
    }
}

#[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
const fn schedule_stage_profile_name(stage: SystemStage) -> &'static str {
    match stage {
        SystemStage::First => "runtime_frame_schedule_stage.First",
        SystemStage::PreUpdate => "runtime_frame_schedule_stage.PreUpdate",
        SystemStage::FixedFirst => "runtime_frame_schedule_stage.FixedFirst",
        SystemStage::FixedUpdate => "runtime_frame_schedule_stage.FixedUpdate",
        SystemStage::FixedPostUpdate => "runtime_frame_schedule_stage.FixedPostUpdate",
        SystemStage::Update => "runtime_frame_schedule_stage.Update",
        SystemStage::PostUpdate => "runtime_frame_schedule_stage.PostUpdate",
        SystemStage::Last => "runtime_frame_schedule_stage.Last",
        SystemStage::RenderExtract => "runtime_frame_schedule_stage.RenderExtract",
    }
}

#[cfg(test)]
mod tests {
    use std::panic::AssertUnwindSafe;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier, Mutex};
    use std::thread;
    use std::time::Duration;

    use crate::core::framework::scene::WorldHandle;
    use crate::core::{CoreRuntime, JobScheduler, TaskPool, TaskPoolDescriptor};
    use crate::plugin::RuntimeExtensionRegistry;
    use crate::scene::components::Name;
    use crate::scene::ecs::{
        CommandsParam, Component, DeferredCommandOperation, DeferredCommandTarget,
        LifecycleEventKind, Resource, SceneSystemThreadAffinity, SystemParamAccess,
    };
    use crate::scene::{EntityId, LevelMetadata, World};

    use super::*;

    mod typed_worker_structural {
        include!("schedule_runner/tests/typed_worker_structural.rs");
    }
    mod worker_callback_order {
        include!("schedule_runner/tests/worker_callback_order.rs");
    }
    #[test]
    fn disjoint_worker_safe_native_systems_overlap_in_the_production_stage_runner() {
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("tests.runtime").unwrap();
        register_timed_external_system(
            &mut registry,
            owner,
            "tests.worker_a",
            "tests.resource_a",
            active.clone(),
            max_active.clone(),
        );
        register_timed_external_system(
            &mut registry,
            owner,
            "tests.worker_b",
            "tests.resource_b",
            active,
            max_active.clone(),
        );
        let core = CoreRuntime::new();
        let level = test_level(registry);

        run_test_stage(&core.handle(), &level);

        let expected = usize::from(core.scheduler().parallelism() > 1) + 1;
        assert_eq!(max_active.load(Ordering::SeqCst), expected);
        let diagnostics = level.with_world(|world| {
            world
                .ecs_frame_performance_diagnostics()
                .native_system_schedule
        });
        assert_eq!(diagnostics.conflict_count(), 0);
        assert_eq!(diagnostics.worker_batch_count(), 1);
        assert_eq!(diagnostics.callback_count(), 2);
        assert_eq!(diagnostics.temporary_control_buffer_count(), 2);
        assert!(diagnostics.temporary_control_buffer_bytes() > 0);
        assert!(diagnostics.callback_p95_ms() >= 30.0);
        assert!(diagnostics.worker_utilization() > 0.0);
    }

    #[test]
    fn conflicting_worker_safe_native_systems_remain_serial() {
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("tests.runtime").unwrap();
        register_timed_external_system(
            &mut registry,
            owner,
            "tests.conflict_a",
            "tests.shared_resource",
            active.clone(),
            max_active.clone(),
        );
        register_timed_external_system(
            &mut registry,
            owner,
            "tests.conflict_b",
            "tests.shared_resource",
            active,
            max_active.clone(),
        );
        let core = CoreRuntime::new();
        let level = test_level(registry);

        run_test_stage(&core.handle(), &level);

        assert_eq!(max_active.load(Ordering::SeqCst), 1);
        let diagnostics = level.with_world(|world| {
            world
                .ecs_frame_performance_diagnostics()
                .native_system_schedule
        });
        assert_eq!(diagnostics.conflict_count(), 1);
        assert_eq!(diagnostics.worker_batch_count(), 2);
        assert_eq!(diagnostics.callback_count(), 2);
    }

    #[test]
    fn main_thread_only_external_system_runs_on_the_schedule_caller() {
        let caller = thread::current().id();
        let observed = Arc::new(Mutex::new(None));
        let observed_for_system = observed.clone();
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("tests.runtime").unwrap();
        registry
            .register_external_native_system(
                owner,
                "tests.main_thread",
                SystemStage::Update,
                SceneSystemThreadAffinity::MainThreadOnly,
                |_world| {
                    let mut access = SystemParamAccess::default();
                    access.add_conservative_world_access();
                    Ok(access)
                },
                move || {
                    let observed = Arc::clone(&observed_for_system);
                    move || {
                        *observed.lock().unwrap() = Some(thread::current().id());
                    }
                },
            )
            .register()
            .unwrap();
        let core = CoreRuntime::new();
        let level = test_level(registry);

        run_test_stage(&core.handle(), &level);

        assert_eq!(*observed.lock().unwrap(), Some(caller));
        let diagnostics = level.with_world(|world| {
            world
                .ecs_frame_performance_diagnostics()
                .native_system_schedule
        });
        assert_eq!(diagnostics.worker_batch_count(), 0);
        assert_eq!(diagnostics.callback_count(), 1);
        assert_eq!(diagnostics.conservative_world_writer_count(), 1);
    }

    #[test]
    fn conservative_world_writer_is_not_dispatched_to_a_worker() {
        let caller = thread::current().id();
        let observed = Arc::new(Mutex::new(None));
        let observed_for_system = observed.clone();
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("tests.runtime").unwrap();
        registry
            .register_external_native_system(
                owner,
                "tests.conservative_world_writer",
                SystemStage::Update,
                SceneSystemThreadAffinity::WorkerSafe,
                |_world| {
                    let mut access = SystemParamAccess::default();
                    access.add_conservative_world_access();
                    Ok(access)
                },
                move || {
                    let observed = Arc::clone(&observed_for_system);
                    move || {
                        *observed.lock().unwrap() = Some(thread::current().id());
                    }
                },
            )
            .register()
            .unwrap();
        let core = CoreRuntime::new();
        let level = test_level(registry);

        run_test_stage(&core.handle(), &level);

        assert_eq!(*observed.lock().unwrap(), Some(caller));
        let diagnostics = level.with_world(|world| {
            world
                .ecs_frame_performance_diagnostics()
                .native_system_schedule
        });
        assert_eq!(diagnostics.worker_batch_count(), 0);
        assert_eq!(diagnostics.callback_count(), 1);
        assert_eq!(diagnostics.conservative_world_writer_count(), 1);
    }

    #[test]
    fn panicking_worker_safe_system_is_restored_before_the_panic_resumes() {
        let panic_once = Arc::new(AtomicBool::new(true));
        let calls = Arc::new(AtomicUsize::new(0));
        let panic_once_for_system = panic_once.clone();
        let calls_for_system = calls.clone();
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("tests.runtime").unwrap();
        registry
            .register_external_native_system(
                owner,
                "tests.panic_once",
                SystemStage::Update,
                SceneSystemThreadAffinity::WorkerSafe,
                |world| {
                    let resource_id = world.external_resource_id("tests.panic_resource");
                    let mut access = SystemParamAccess::default();
                    access
                        .add_resource_write(resource_id)
                        .map_err(|error| error.to_string())?;
                    Ok(access)
                },
                move || {
                    let calls = Arc::clone(&calls_for_system);
                    let panic_once = Arc::clone(&panic_once_for_system);
                    move || {
                        calls.fetch_add(1, Ordering::SeqCst);
                        if panic_once.swap(false, Ordering::SeqCst) {
                            panic!("intentional worker callback panic");
                        }
                    }
                },
            )
            .register()
            .unwrap();
        let core = CoreRuntime::new();
        let level = test_level(registry);

        let first =
            std::panic::catch_unwind(AssertUnwindSafe(|| run_test_stage(&core.handle(), &level)));
        assert!(first.is_err());
        run_test_stage(&core.handle(), &level);

        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn panicking_worker_command_restores_native_system_before_the_panic_resumes() {
        let panic_once = Arc::new(AtomicBool::new(true));
        let calls = Arc::new(AtomicUsize::new(0));
        let panic_once_for_system = panic_once.clone();
        let calls_for_system = calls.clone();
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("tests.runtime").unwrap();
        registry
            .register_external_native_command_system(
                owner,
                "tests.command_panic_once",
                SystemStage::Update,
                SceneSystemThreadAffinity::WorkerSafe,
                |_world| Ok(SystemParamAccess::default()),
                move || {
                    let calls = Arc::clone(&calls_for_system);
                    let panic_once = Arc::clone(&panic_once_for_system);
                    move |commands| {
                        calls.fetch_add(1, Ordering::SeqCst);
                        let panic_once_for_command = Arc::clone(&panic_once);
                        commands.push(move |_world: &mut World| {
                            if panic_once_for_command.swap(false, Ordering::SeqCst) {
                                panic!("intentional worker command panic");
                            }
                        });
                    }
                },
            )
            .with_command_capacity(1)
            .register()
            .unwrap();
        let core = CoreRuntime::new();
        let level = test_level(registry);

        let first =
            std::panic::catch_unwind(AssertUnwindSafe(|| run_test_stage(&core.handle(), &level)));
        assert!(first.is_err());
        run_test_stage(&core.handle(), &level);

        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn panicking_worker_callback_discards_its_local_commands_before_retry() {
        let panic_once = Arc::new(AtomicBool::new(true));
        let calls = Arc::new(AtomicUsize::new(0));
        let published = Arc::new(AtomicUsize::new(0));
        let panic_once_for_system = Arc::clone(&panic_once);
        let calls_for_system = Arc::clone(&calls);
        let published_for_system = Arc::clone(&published);
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("tests.runtime").unwrap();
        registry
            .register_external_native_command_system(
                owner,
                "tests.callback_panic_discards_local_commands",
                SystemStage::Update,
                SceneSystemThreadAffinity::WorkerSafe,
                |_world| Ok(SystemParamAccess::default()),
                move || {
                    let panic_once = Arc::clone(&panic_once_for_system);
                    let calls = Arc::clone(&calls_for_system);
                    let published = Arc::clone(&published_for_system);
                    move |commands| {
                        calls.fetch_add(1, Ordering::SeqCst);
                        let published = Arc::clone(&published);
                        commands.push(move |_world: &mut World| {
                            published.fetch_add(1, Ordering::SeqCst);
                        });
                        if panic_once.swap(false, Ordering::SeqCst) {
                            panic!("intentional worker callback panic after enqueue");
                        }
                    }
                },
            )
            .with_command_capacity(1)
            .register()
            .unwrap();
        let core = CoreRuntime::new();
        let level = test_level(registry);

        let first =
            std::panic::catch_unwind(AssertUnwindSafe(|| run_test_stage(&core.handle(), &level)));
        assert!(first.is_err());
        assert_eq!(published.load(Ordering::SeqCst), 0);

        run_test_stage(&core.handle(), &level);

        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(published.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn panicking_worker_callback_discards_sibling_worker_commands_before_retry() {
        let panic_once = Arc::new(AtomicBool::new(true));
        let first_window = Arc::new(AtomicBool::new(true));
        let first_window_barrier = Arc::new(Barrier::new(2));
        let published = Arc::new(AtomicUsize::new(0));
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("tests.runtime").unwrap();

        let first_window_for_sibling = Arc::clone(&first_window);
        let first_window_barrier_for_sibling = Arc::clone(&first_window_barrier);
        let published_for_sibling = Arc::clone(&published);
        registry
            .register_external_native_command_system(
                owner,
                "tests.sibling_worker_command",
                SystemStage::Update,
                SceneSystemThreadAffinity::WorkerSafe,
                |_world| Ok(SystemParamAccess::default()),
                move || {
                    let first_window = Arc::clone(&first_window_for_sibling);
                    let first_window_barrier = Arc::clone(&first_window_barrier_for_sibling);
                    let published = Arc::clone(&published_for_sibling);
                    move |commands| {
                        let published = Arc::clone(&published);
                        commands.push(move |_world: &mut World| {
                            published.fetch_add(1, Ordering::SeqCst);
                        });
                        if first_window.load(Ordering::SeqCst) {
                            first_window_barrier.wait();
                        }
                    }
                },
            )
            .with_order(10)
            .with_command_capacity(1)
            .register()
            .unwrap();

        let panic_once_for_system = Arc::clone(&panic_once);
        let first_window_for_panic = Arc::clone(&first_window);
        let first_window_barrier_for_panic = Arc::clone(&first_window_barrier);
        registry
            .register_external_native_command_system(
                owner,
                "tests.panic_after_sibling_enqueue",
                SystemStage::Update,
                SceneSystemThreadAffinity::WorkerSafe,
                |_world| Ok(SystemParamAccess::default()),
                move || {
                    let panic_once = Arc::clone(&panic_once_for_system);
                    let first_window = Arc::clone(&first_window_for_panic);
                    let first_window_barrier = Arc::clone(&first_window_barrier_for_panic);
                    move |_commands| {
                        if panic_once.swap(false, Ordering::SeqCst) {
                            first_window_barrier.wait();
                            first_window.store(false, Ordering::SeqCst);
                            panic!("intentional worker callback panic after sibling enqueue");
                        }
                    }
                },
            )
            .with_order(20)
            .register()
            .unwrap();

        let level = test_level(registry);
        let scheduler = JobScheduler::from_pool(TaskPool::new(
            TaskPoolDescriptor::compute().with_worker_threads(2),
        ));
        let mut dispatches = vec![
            WorkerDispatch {
                id: "tests.sibling_worker_command",
                key: DeferredSystemKey::compiled(
                    SystemStage::Update.rank(),
                    10,
                    "tests.sibling_worker_command",
                ),
            },
            WorkerDispatch {
                id: "tests.panic_after_sibling_enqueue",
                key: DeferredSystemKey::compiled(
                    SystemStage::Update.rank(),
                    20,
                    "tests.panic_after_sibling_enqueue",
                ),
            },
        ];

        let first = std::panic::catch_unwind(AssertUnwindSafe(|| {
            flush_worker_batch(&scheduler, &level, &mut dispatches)
        }));
        assert!(first.is_err());
        assert_eq!(published.load(Ordering::SeqCst), 0);

        dispatches = vec![
            WorkerDispatch {
                id: "tests.sibling_worker_command",
                key: DeferredSystemKey::compiled(
                    SystemStage::Update.rank(),
                    10,
                    "tests.sibling_worker_command",
                ),
            },
            WorkerDispatch {
                id: "tests.panic_after_sibling_enqueue",
                key: DeferredSystemKey::compiled(
                    SystemStage::Update.rank(),
                    20,
                    "tests.panic_after_sibling_enqueue",
                ),
            },
        ];
        flush_worker_batch(&scheduler, &level, &mut dispatches)
            .expect("clean retry must reuse the restored worker systems");

        assert_eq!(published.load(Ordering::SeqCst), 1);
    }

    fn register_timed_external_system(
        registry: &mut RuntimeExtensionRegistry,
        owner: crate::plugin::PluginModuleId,
        system_id: &'static str,
        resource_id: &'static str,
        active: Arc<AtomicUsize>,
        max_active: Arc<AtomicUsize>,
    ) {
        registry
            .register_external_native_system(
                owner,
                system_id,
                SystemStage::Update,
                SceneSystemThreadAffinity::WorkerSafe,
                move |world| {
                    let resource_id = world.external_resource_id(resource_id);
                    let mut access = SystemParamAccess::default();
                    access
                        .add_resource_write(resource_id)
                        .map_err(|error| error.to_string())?;
                    Ok(access)
                },
                move || {
                    let active = Arc::clone(&active);
                    let max_active = Arc::clone(&max_active);
                    move || {
                        let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                        max_active.fetch_max(current, Ordering::SeqCst);
                        thread::sleep(Duration::from_millis(30));
                        active.fetch_sub(1, Ordering::SeqCst);
                    }
                },
            )
            .register()
            .unwrap();
    }

    fn test_level(mut registry: RuntimeExtensionRegistry) -> LevelSystem {
        let mut world = World::empty();
        registry.apply_to_world(&mut world).unwrap();
        LevelSystem::new(
            WorldHandle::new(1),
            Arc::new(Mutex::new(world)),
            LevelMetadata::default(),
        )
    }

    fn run_test_stage(core: &CoreHandle, level: &LevelSystem) {
        let schedule = level.with_world(|world| world.schedule().stage_plan());
        SceneScheduleRunner::run_stage(
            core,
            level,
            SystemStage::Update,
            0.0,
            schedule.internal_systems_for_stage(SystemStage::Update),
            schedule.native_steps_for_stage(SystemStage::Update),
            schedule.native_conflict_graph_for_stage(SystemStage::Update),
        )
        .unwrap();
    }
}

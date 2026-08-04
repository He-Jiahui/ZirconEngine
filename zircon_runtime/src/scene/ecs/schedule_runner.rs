use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::time::Instant;

use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle, JobScheduler};
use crate::scene::ecs::{
    BoxedSceneSystem, InternalSceneSystem, NativeSystemCallbackTiming, SceneSystemDescriptor,
    ScheduleConflictGraph, ScheduledSceneStep, ScheduledSceneStepRef, SystemStage,
};
use crate::scene::LevelSystem;
use crate::scene::{SceneRuntimeHookContext, SceneRuntimeHookRegistration};

pub(crate) struct SceneScheduleRunner;

impl SceneScheduleRunner {
    pub(crate) fn run_stage(
        core: &CoreHandle,
        level: &LevelSystem,
        stage: SystemStage,
        delta_seconds: Real,
        internal_systems: &[SceneSystemDescriptor],
        native_steps: &[ScheduledSceneStep],
        native_conflicts: &ScheduleConflictGraph,
        hooks: &[SceneRuntimeHookRegistration],
    ) -> Result<(), CoreError> {
        crate::profile_scope!("runtime", "frame", schedule_stage_profile_name(stage),);

        level.with_world_mut(|world| world.set_scene_system_flush_deferred(true));
        level.with_world_mut(|world| {
            world.record_native_system_conflicts(native_conflicts.edges().len())
        });

        let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), CoreError> {
            let mut worker_batch = Vec::new();
            for step in ScheduledSceneStep::iter_sorted_for_stage(
                stage,
                internal_systems,
                native_steps,
                hooks,
            ) {
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
                        worker_safe,
                        conservative_world_writer,
                        ..
                    } => {
                        if worker_safe {
                            if worker_batch
                                .iter()
                                .any(|other| native_conflicts.systems_conflict(other, id))
                            {
                                flush_worker_batch(core.scheduler(), level, &mut worker_batch)?;
                            }
                            worker_batch.push(id);
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
                    ScheduledSceneStepRef::Hook(hook) => {
                        flush_worker_batch(core.scheduler(), level, &mut worker_batch)?;
                        hook.run(SceneRuntimeHookContext::new(core, level, delta_seconds))?;
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
    system_ids: &mut Vec<&str>,
) -> Result<(), CoreError> {
    if system_ids.is_empty() {
        return Ok(());
    }
    let ready_at = Instant::now();
    let mut systems = level
        .with_world_mut(|world| world.take_worldless_native_scene_systems(system_ids))
        .expect("compiled worker-safe scene systems must remain registered for the stage");
    let mut timings = vec![NativeSystemCallbackTiming::default(); systems.len()];
    let batch_started_at = Instant::now();
    let result = catch_unwind(AssertUnwindSafe(|| {
        run_worldless_systems(scheduler, &mut systems, &mut timings, ready_at)
    }));
    let command_buffer_result: Result<(), crate::scene::ecs::WorkerCommandBufferMergeError> =
        if result.is_ok() {
            let mut command_buffers = systems
                .iter_mut()
                .filter_map(|system| system.worker_command_buffer_mut())
                .collect::<Vec<_>>();
            let has_worker_commands = !command_buffers.is_empty();
            level.with_world_mut(|world| {
                world.merge_worker_command_buffers(&mut command_buffers)?;
                if has_worker_commands {
                    world.apply_deferred();
                }
                Ok(())
            })
        } else {
            Ok(())
        };
    let batch_elapsed = batch_started_at.elapsed();
    level.with_world_mut(|world| {
        world.restore_worldless_native_scene_systems(systems);
        world.record_native_system_worker_batch(&timings, batch_elapsed, scheduler.parallelism());
    });
    system_ids.clear();
    if let Err(payload) = result {
        resume_unwind(payload);
    }
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
        ([system], [timing]) => {
            let started_at = Instant::now();
            timing.ready_delay = started_at.saturating_duration_since(ready_at);
            let result = catch_unwind(AssertUnwindSafe(|| system.run_without_world()));
            timing.callback = started_at.elapsed();
            if let Err(payload) = result {
                resume_unwind(payload);
            }
        }
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
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    use crate::core::framework::scene::WorldHandle;
    use crate::core::CoreRuntime;
    use crate::plugin::RuntimeExtensionRegistry;
    use crate::scene::ecs::{Resource, SceneSystemThreadAffinity, SystemParamAccess};
    use crate::scene::{LevelMetadata, World};

    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct WorkerCommandCallbackOrder(Vec<u8>);

    impl Resource for WorkerCommandCallbackOrder {}

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
    fn worker_command_callbacks_merge_once_in_compiled_order_before_apply() {
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("tests.runtime").unwrap();
        registry
            .register_external_native_command_system(
                owner,
                "tests.worker_command.z",
                SystemStage::Update,
                SceneSystemThreadAffinity::WorkerSafe,
                |_world| Ok(SystemParamAccess::default()),
                |commands| {
                    commands.push(|world: &mut World| {
                        world
                            .get_resource_mut::<WorkerCommandCallbackOrder>()
                            .expect("earlier worker command callbacks should apply first")
                            .0
                            .push(3);
                    });
                },
            )
            .with_order(30)
            .with_command_capacity(1)
            .register()
            .unwrap();
        registry
            .register_external_native_command_system(
                owner,
                "tests.worker_command.b",
                SystemStage::Update,
                SceneSystemThreadAffinity::WorkerSafe,
                |_world| Ok(SystemParamAccess::default()),
                |commands| {
                    commands.push(|world: &mut World| {
                        world
                            .get_resource_mut::<WorkerCommandCallbackOrder>()
                            .expect("first worker command callback should apply first")
                            .0
                            .push(2);
                    });
                },
            )
            .with_order(10)
            .with_command_capacity(1)
            .register()
            .unwrap();
        registry
            .register_external_native_command_system(
                owner,
                "tests.worker_command.a",
                SystemStage::Update,
                SceneSystemThreadAffinity::WorkerSafe,
                |_world| Ok(SystemParamAccess::default()),
                |commands| {
                    commands.push(|world: &mut World| {
                        world.insert_resource(WorkerCommandCallbackOrder(vec![1]));
                    });
                },
            )
            .with_order(10)
            .with_command_capacity(1)
            .register()
            .unwrap();
        let core = CoreRuntime::new();
        let level = test_level(registry);

        run_test_stage(&core.handle(), &level);

        assert_eq!(
            level.with_world(|world| world.get_resource::<WorkerCommandCallbackOrder>().cloned()),
            Some(WorkerCommandCallbackOrder(vec![1, 2, 3]))
        );
        let diagnostics = level.with_world(|world| {
            world
                .ecs_frame_performance_diagnostics()
                .native_system_schedule
        });
        assert_eq!(diagnostics.worker_batch_count(), 1);
        assert_eq!(diagnostics.callback_count(), 3);
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
                    *observed_for_system.lock().unwrap() = Some(thread::current().id());
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
                    calls_for_system.fetch_add(1, Ordering::SeqCst);
                    if panic_once_for_system.swap(false, Ordering::SeqCst) {
                        panic!("intentional worker callback panic");
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
                    let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                    max_active.fetch_max(current, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(30));
                    active.fetch_sub(1, Ordering::SeqCst);
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
            &[],
        )
        .unwrap();
    }
}

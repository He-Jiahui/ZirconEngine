use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::time::Instant;

use crate::core::{CoreError, CoreHandle, JobScheduler};
use crate::scene::LevelSystem;
use crate::scene::ecs::{
    BoxedSceneSystem, DeferredSystemKey, InternalSceneSystem, NativeSystemCallbackTiming,
    SceneStageTickContexts, SceneSystemDescriptor, SceneSystemTickPolicy, ScheduleConflictGraph,
    ScheduledSceneStep, ScheduledSceneStepRef, SystemStage,
};

pub(crate) struct SceneScheduleRunner;

#[derive(Debug)]
pub(crate) struct SceneStageRunError {
    system_id: Option<String>,
    source: CoreError,
}

impl SceneStageRunError {
    fn runtime_system(system_id: &str, source: CoreError) -> Self {
        Self {
            system_id: Some(system_id.to_owned()),
            source,
        }
    }

    pub(crate) fn into_parts(self) -> (Option<String>, CoreError) {
        (self.system_id, self.source)
    }
}

impl From<CoreError> for SceneStageRunError {
    fn from(source: CoreError) -> Self {
        Self {
            system_id: None,
            source,
        }
    }
}

struct WorkerDispatch<'a> {
    id: &'a str,
    key: DeferredSystemKey,
}

impl SceneScheduleRunner {
    pub(crate) fn run_stage(
        core: &CoreHandle,
        level: &LevelSystem,
        stage: SystemStage,
        tick_contexts: SceneStageTickContexts,
        virtual_time_paused: bool,
        internal_systems: &[SceneSystemDescriptor],
        native_steps: &[ScheduledSceneStep],
        native_conflicts: &ScheduleConflictGraph,
    ) -> Result<(), SceneStageRunError> {
        crate::profile_scope!("runtime", "frame", schedule_stage_profile_name(stage),);
        let deferred_apply_allowed = !virtual_time_paused;

        level.with_world_mut(|world| world.set_scene_system_flush_deferred(true));
        level.with_world_mut(|world| {
            world.record_native_system_conflicts(native_conflicts.edges().len())
        });

        let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), SceneStageRunError> {
            let mut worker_batch = Vec::new();
            for step in
                ScheduledSceneStep::iter_sorted_for_stage(stage, internal_systems, native_steps)
            {
                match step {
                    ScheduledSceneStepRef::Internal(system) => {
                        if should_skip_for_pause(virtual_time_paused, system.system().tick_policy())
                        {
                            continue;
                        }
                        flush_worker_batch(
                            core.scheduler(),
                            level,
                            &mut worker_batch,
                            deferred_apply_allowed,
                        )?;
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
                        tick_policy,
                        worker_safe,
                        conservative_world_writer,
                    } => {
                        if should_skip_for_pause(virtual_time_paused, tick_policy) {
                            continue;
                        }
                        if worker_safe {
                            if worker_batch
                                .iter()
                                .any(|other| native_conflicts.systems_conflict(other.id, id))
                            {
                                flush_worker_batch(
                                    core.scheduler(),
                                    level,
                                    &mut worker_batch,
                                    deferred_apply_allowed,
                                )?;
                            }
                            worker_batch.push(WorkerDispatch {
                                id,
                                key: DeferredSystemKey::compiled(step_stage.rank(), order, id),
                            });
                        } else {
                            flush_worker_batch(
                                core.scheduler(),
                                level,
                                &mut worker_batch,
                                deferred_apply_allowed,
                            )?;
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
                    ScheduledSceneStepRef::Runtime {
                        id, tick_policy, ..
                    } => {
                        if should_skip_for_pause(virtual_time_paused, tick_policy) {
                            continue;
                        }
                        flush_worker_batch(
                            core.scheduler(),
                            level,
                            &mut worker_batch,
                            deferred_apply_allowed,
                        )?;
                        level
                            .run_runtime_scene_system(
                                core,
                                id,
                                tick_contexts.for_domain(tick_policy.clock_domain()),
                            )
                            .map_err(|source| SceneStageRunError::runtime_system(id, source))?;
                        if deferred_apply_allowed {
                            level.with_world_mut(|world| world.apply_deferred());
                        }
                    }
                    ScheduledSceneStepRef::ApplyDeferred { tick_policy, .. } => {
                        if should_skip_for_pause(virtual_time_paused, tick_policy) {
                            continue;
                        }
                        flush_worker_batch(
                            core.scheduler(),
                            level,
                            &mut worker_batch,
                            deferred_apply_allowed,
                        )?;
                        level.with_world_mut(|world| world.apply_deferred());
                    }
                }
            }
            flush_worker_batch(
                core.scheduler(),
                level,
                &mut worker_batch,
                deferred_apply_allowed,
            )?;

            Ok(())
        }));
        let stage_succeeded = matches!(&result, Ok(Ok(())));
        level.with_world_mut(|world| {
            world.set_scene_system_flush_deferred(false);
            if stage_succeeded && !virtual_time_paused {
                world.flush_pending_scene_systems_for_stage(stage);
            }
        });
        match result {
            Ok(result) => result,
            Err(payload) => resume_unwind(payload),
        }
    }
}

fn should_skip_for_pause(virtual_time_paused: bool, tick_policy: SceneSystemTickPolicy) -> bool {
    virtual_time_paused && !tick_policy.runs_when_virtual_paused()
}

fn flush_worker_batch(
    scheduler: &JobScheduler,
    level: &LevelSystem,
    dispatches: &mut Vec<WorkerDispatch<'_>>,
    deferred_apply_allowed: bool,
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
        if deferred_apply_allowed {
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
        } else {
            for buffer in &mut command_buffers {
                buffer.discard_pending();
            }
            Ok(())
        }
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
#[path = "schedule_runner/tests/mod.rs"]
mod tests;

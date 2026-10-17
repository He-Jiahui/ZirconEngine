use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use web_time::Instant;

use super::schedule_execution::schedule_owner_changed_error;

use crate::core::{CoreError, CoreHandle, JobScheduler};
use crate::scene::ecs::{
    BoxedSceneSystem, DeferredSystemKey, InternalSceneSystem, NativeSystemCallbackTiming,
    SceneStageTickContexts, SceneSystemTickPolicy, ScheduleExecution, ScheduledSceneStep,
    ScheduledSceneStepRef, SystemStage,
};
use crate::scene::LevelSystem;

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
        execution: &ScheduleExecution,
    ) -> Result<(), SceneStageRunError> {
        crate::profile_scope!("runtime", "frame", schedule_stage_profile_name(stage),);
        let plan_owner = execution.plan();
        let deferred_apply_allowed = !virtual_time_paused;
        let internal_systems = plan_owner.internal_systems_for_stage(stage);
        let native_steps = plan_owner.native_steps_for_stage(stage);
        let native_conflicts = plan_owner.native_conflict_graph_for_stage(stage);

        with_schedule_world(level, execution, |world| {
            world.set_scene_system_flush_deferred(true);
            world.record_native_system_conflicts(native_conflicts.edges().len());
        })?;

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
                            execution,
                        )?;
                        with_schedule_world(level, execution, |world| {
                            world.run_internal_scene_system(system.system())
                        })?;
                        if !matches!(
                            system.system(),
                            InternalSceneSystem::ApplyDeferred | InternalSceneSystem::UpdateEvents
                        ) {
                            with_schedule_world(level, execution, |world| world.apply_deferred())?;
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
                                    execution,
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
                                execution,
                            )?;
                            with_schedule_world(level, execution, |world| {
                                let started_at = Instant::now();
                                let result = catch_unwind(AssertUnwindSafe(|| {
                                    world.run_native_scene_system(id)
                                }));
                                if execution.is_bound_to(world) {
                                    world.record_native_system_main_callback(
                                        started_at.elapsed(),
                                        conservative_world_writer,
                                    );
                                }
                                if let Err(payload) = result {
                                    resume_unwind(payload);
                                }
                            })?;
                            with_schedule_world(level, execution, |_| ())?;
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
                            execution,
                        )?;
                        level
                            .run_runtime_scene_system(
                                core,
                                id,
                                tick_contexts.for_domain(tick_policy.clock_domain()),
                                execution,
                            )
                            .map_err(|source| SceneStageRunError::runtime_system(id, source))?;
                        if deferred_apply_allowed {
                            with_schedule_world(level, execution, |world| world.apply_deferred())?;
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
                            execution,
                        )?;
                        with_schedule_world(level, execution, |world| world.apply_deferred())?;
                    }
                }
            }
            flush_worker_batch(
                core.scheduler(),
                level,
                &mut worker_batch,
                deferred_apply_allowed,
                execution,
            )?;

            Ok(())
        }));
        let stage_succeeded = matches!(&result, Ok(Ok(())));
        let cleanup = level.with_world_mut_if_schedule_execution(execution, |world| {
            world.set_scene_system_flush_deferred(false);
            if stage_succeeded && !virtual_time_paused {
                world.flush_pending_scene_systems_for_stage(stage);
            }
        });
        match result {
            Ok(Err(error)) => Err(error),
            Err(payload) => resume_unwind(payload),
            Ok(Ok(())) => cleanup
                .ok_or_else(schedule_owner_changed_error)
                .map_err(SceneStageRunError::from),
        }
    }
}

fn should_skip_for_pause(virtual_time_paused: bool, tick_policy: SceneSystemTickPolicy) -> bool {
    virtual_time_paused && !tick_policy.runs_when_virtual_paused()
}

fn with_schedule_world<R>(
    level: &LevelSystem,
    execution: &ScheduleExecution,
    write: impl FnOnce(&mut crate::scene::World) -> R,
) -> Result<R, CoreError> {
    level
        .with_world_mut_if_schedule_execution(execution, write)
        .ok_or_else(schedule_owner_changed_error)
}

fn flush_worker_batch(
    scheduler: &JobScheduler,
    level: &LevelSystem,
    dispatches: &mut Vec<WorkerDispatch<'_>>,
    deferred_apply_allowed: bool,
    execution: &ScheduleExecution,
) -> Result<(), CoreError> {
    if dispatches.is_empty() {
        return Ok(());
    }
    let ready_at = Instant::now();
    let system_ids = dispatches
        .iter()
        .map(|dispatch| dispatch.id)
        .collect::<Vec<_>>();
    let systems = with_schedule_world(level, execution, |world| {
        let return_owner = world.schedule().executable_return_handle();
        (
            world.take_worldless_native_scene_systems(&system_ids),
            return_owner,
        )
    })?;
    let (systems, return_owner) = systems;
    let mut systems =
        systems.expect("compiled worker-safe scene systems must remain registered for the stage");
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
            with_schedule_world(level, execution, |world| {
                world
                    .merge_worker_command_buffers(&mut command_buffers)
                    .map_err(|error| {
                        CoreError::Initialization(
                            "SceneScheduleRunner worker command buffer merge".to_string(),
                            error.to_string(),
                        )
                    })?;
                if has_worker_commands {
                    let apply_result = catch_unwind(AssertUnwindSafe(|| world.apply_deferred()));
                    if execution.is_bound_to(world) {
                        world.reclaim_worker_command_buffers(&mut command_buffers);
                    }
                    if let Err(payload) = apply_result {
                        resume_unwind(payload);
                    }
                    if !execution.is_bound_to(world) {
                        return Err(schedule_owner_changed_error());
                    }
                }
                Ok(())
            })?
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
    // 先归还系统并清空调度条目，再传播批次结果，使捕获异常后的再次调度能取得同一系统；已提交的 World 变更不会在此回滚。
    let batch_elapsed = batch_started_at.elapsed();
    let mut systems = Some(systems);
    let restored = level.with_world_mut_if_schedule_execution(execution, |world| {
        world.restore_worldless_native_scene_systems(systems.take().unwrap());
        world.record_native_system_worker_batch(
            &timings,
            batch_elapsed,
            scheduler.parallelism(),
            temporary_control_buffer_count,
            temporary_control_buffer_bytes,
        );
    });
    if let Some(systems) = systems {
        for system in systems {
            return_owner.native(system);
        }
    }
    dispatches.clear();
    match result {
        Ok(Err(error)) => Err(error),
        Err(payload) => resume_unwind(payload),
        Ok(Ok(())) => restored
            .ok_or_else(schedule_owner_changed_error)
            .map(|()| ()),
    }
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

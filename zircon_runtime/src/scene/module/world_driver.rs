use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::{CoreError, CoreHandle, FrameTimeSnapshot};
use crate::scene::ecs::{
    SceneScheduleRunner, SceneStageRunError, SceneStageTickContexts, SystemStage, SystemTickContext,
};
use crate::scene::world_time::{WorldFixedStep, WorldTimeSnapshot};
use crate::scene::{
    FixedStepFailurePhase, FixedStepFailureReceipt, LevelSystem, LevelTickError, SimulationTickId,
    World, WorldRuntimeExtensionPlan,
};

#[derive(Debug, Default)]
pub struct WorldDriver {
    runtime_extensions: Mutex<Arc<WorldRuntimeExtensionPlan>>,
}

impl WorldDriver {
    pub fn install_world_runtime_extension_plan(
        &self,
        contribution: WorldRuntimeExtensionPlan,
    ) -> Result<(), CoreError> {
        let mut plan = lock_poison_recovered(&self.runtime_extensions);
        let candidate = plan.as_ref().try_merge(contribution).map_err(|error| {
            CoreError::Initialization("WorldDriver".to_string(), error.to_string())
        })?;
        *plan = Arc::new(candidate);
        Ok(())
    }

    pub fn apply_world_runtime_extensions(&self, world: &mut World) -> Result<(), CoreError> {
        let plan = self.runtime_extension_plan_snapshot();
        plan.apply_to_world(world).map_err(|error| {
            CoreError::Initialization("WorldDriver".to_string(), error.to_string())
        })
    }

    fn runtime_extension_plan_snapshot(&self) -> Arc<WorldRuntimeExtensionPlan> {
        Arc::clone(&lock_poison_recovered(&self.runtime_extensions))
    }

    pub(crate) fn tick_level(
        &self,
        core: &CoreHandle,
        level: &LevelSystem,
        snapshot: FrameTimeSnapshot,
    ) -> Result<(), LevelTickError> {
        let world_time = level.advance_world_time(snapshot)?;
        let fixed_step_plan = world_time.fixed_step_plan();
        let world_generation = level.world_generation();
        let schedule = level.with_world(|world| world.schedule().stage_plan());
        level.with_world_mut(|world| {
            world.reclaim_dropped_runtime_event_mirrors();
            world.reset_ecs_frame_performance_diagnostics();
        });
        for stage in schedule.stages() {
            if *stage == SystemStage::FixedFirst {
                for committed_steps in 0..fixed_step_plan.step_count {
                    let mut active_step = ActiveFixedStep::begin(level, world_generation)?;
                    let tick = active_step.step()?.id();
                    let execution = (|| -> Result<(), FixedStageExecutionError> {
                        for fixed_stage in SystemStage::FIXED_LOOP {
                            ensure_fixed_step_world_generation(
                                level,
                                world_generation,
                                fixed_stage,
                            )
                            .map_err(|source| {
                                FixedStageExecutionError::unattributed(fixed_stage, source)
                            })?;
                            let active_fixed_step = active_step.step().map_err(|source| {
                                FixedStageExecutionError::unattributed(fixed_stage, source)
                            })?;
                            let tick_contexts = stage_tick_contexts(
                                fixed_stage,
                                world_time,
                                world_generation,
                                Some(active_fixed_step),
                                std::time::Duration::ZERO,
                            )
                            .map_err(|source| {
                                FixedStageExecutionError::unattributed(fixed_stage, source)
                            })?;
                            run_stage(core, level, fixed_stage, tick_contexts, false, &schedule)
                                .map_err(|error| {
                                    FixedStageExecutionError::from_stage(fixed_stage, error)
                                })?;
                            ensure_fixed_step_world_generation(
                                level,
                                world_generation,
                                fixed_stage,
                            )
                            .map_err(|source| {
                                FixedStageExecutionError::unattributed(fixed_stage, source)
                            })?;
                        }
                        Ok(())
                    })();
                    if let Err(error) = execution {
                        if let Err(abort_error) = active_step.abort() {
                            return Err(LevelTickError::from(CoreError::Initialization(
                                "WorldDriver fixed-step execution rollback".to_string(),
                                format!("{}; abort failed: {abort_error}", error.source),
                            )));
                        }
                        return Err(fixed_step_failure(
                            level,
                            FixedStepFailurePhase::Stage(error.stage),
                            tick,
                            error.system_id,
                            committed_steps,
                            error.source,
                        ));
                    }
                    match active_step.commit(world_generation) {
                        Ok(()) => {}
                        Err(FixedStepCommitError::Rejected(source)) => {
                            return Err(fixed_step_failure(
                                level,
                                FixedStepFailurePhase::Commit,
                                tick,
                                None,
                                committed_steps,
                                source,
                            ));
                        }
                        Err(FixedStepCommitError::Rollback(source)) => {
                            return Err(LevelTickError::from(source));
                        }
                    }
                }
                continue;
            }

            if stage.is_fixed_loop() {
                continue;
            }

            run_stage(
                core,
                level,
                *stage,
                stage_tick_contexts(
                    *stage,
                    world_time,
                    world_generation,
                    None,
                    level.world_time().fixed_time().elapsed(),
                )?,
                world_time.virtual_time_paused(),
                &schedule,
            )
            .map_err(|error| LevelTickError::from(error.into_parts().1))?;
        }

        level.with_world(|world| {
            world
                .ecs_frame_performance_diagnostics()
                .publish(core, world_time.outer_frame_index());
        });

        Ok(())
    }
}

struct ActiveFixedStep<'a> {
    level: &'a LevelSystem,
    step: Option<WorldFixedStep>,
}

impl<'a> ActiveFixedStep<'a> {
    fn begin(level: &'a LevelSystem, world_generation: u64) -> Result<Self, CoreError> {
        let step = level
            .begin_fixed_step(world_generation)
            .map_err(|error| fixed_step_error("begin", error))?;
        Ok(Self {
            level,
            step: Some(step),
        })
    }

    fn step(&self) -> Result<&WorldFixedStep, CoreError> {
        self.step.as_ref().ok_or_else(|| {
            fixed_step_invariant_error(
                "active transaction was missing before fixed-stage execution",
            )
        })
    }

    fn commit(&mut self, expected_world_generation: u64) -> Result<(), FixedStepCommitError> {
        let step = self.step.as_ref().ok_or_else(|| {
            FixedStepCommitError::Rollback(fixed_step_invariant_error(
                "active transaction was already settled before commit",
            ))
        })?;
        if let Err(error) = self
            .level
            .commit_fixed_step(expected_world_generation, step)
        {
            let commit_error = fixed_step_error("commit", error);
            return match self.abort() {
                Ok(()) => Err(FixedStepCommitError::Rejected(commit_error)),
                Err(abort_error) => Err(FixedStepCommitError::Rollback(CoreError::Initialization(
                    "WorldDriver fixed-step commit rollback".to_string(),
                    format!("{commit_error}; abort failed: {abort_error}"),
                ))),
            };
        }
        let _settled = self.step.take();
        Ok(())
    }

    fn abort(&mut self) -> Result<(), CoreError> {
        let step = self.step.take().ok_or_else(|| {
            fixed_step_invariant_error("active transaction was already settled before abort")
        })?;
        self.level
            .abort_fixed_step(step)
            .map_err(|error| fixed_step_error("abort", error))
    }
}

enum FixedStepCommitError {
    Rejected(CoreError),
    Rollback(CoreError),
}

struct FixedStageExecutionError {
    stage: SystemStage,
    system_id: Option<String>,
    source: CoreError,
}

impl FixedStageExecutionError {
    fn unattributed(stage: SystemStage, source: CoreError) -> Self {
        Self {
            stage,
            system_id: None,
            source,
        }
    }

    fn from_stage(stage: SystemStage, error: SceneStageRunError) -> Self {
        let (system_id, source) = error.into_parts();
        Self {
            stage,
            system_id,
            source,
        }
    }
}

impl Drop for ActiveFixedStep<'_> {
    fn drop(&mut self) {
        if let Some(step) = self.step.take() {
            let _ = self.level.abort_fixed_step(step);
        }
    }
}

fn fixed_step_error(operation: &str, error: impl std::fmt::Display) -> CoreError {
    CoreError::Initialization(
        format!("WorldDriver fixed-step {operation}"),
        error.to_string(),
    )
}

fn fixed_step_invariant_error(reason: &str) -> CoreError {
    CoreError::Initialization(
        "WorldDriver fixed-step invariant".to_string(),
        reason.to_string(),
    )
}

fn fixed_step_failure(
    level: &LevelSystem,
    phase: FixedStepFailurePhase,
    tick: SimulationTickId,
    system_id: Option<String>,
    committed_steps: u32,
    source: CoreError,
) -> LevelTickError {
    LevelTickError::fixed_step(
        FixedStepFailureReceipt::new(
            phase,
            tick,
            system_id,
            committed_steps,
            level.world_time().fixed_time().overstep(),
            level.world_generation(),
        ),
        source,
    )
}

fn ensure_fixed_step_world_generation(
    level: &LevelSystem,
    expected: u64,
    stage: SystemStage,
) -> Result<(), CoreError> {
    let actual = level.world_generation();
    if actual == expected {
        return Ok(());
    }
    Err(CoreError::Initialization(
        "WorldDriver fixed-step world generation".to_string(),
        format!("World generation changed during {stage:?}: expected {expected}, found {actual}"),
    ))
}

fn lock_poison_recovered<T>(lock: &Mutex<T>) -> MutexGuard<'_, T> {
    lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn run_stage(
    core: &CoreHandle,
    level: &LevelSystem,
    stage: SystemStage,
    tick_contexts: SceneStageTickContexts,
    virtual_time_paused: bool,
    schedule: &crate::scene::ecs::SceneScheduleStagePlan,
) -> Result<(), SceneStageRunError> {
    SceneScheduleRunner::run_stage(
        core,
        level,
        stage,
        tick_contexts,
        virtual_time_paused,
        schedule.internal_systems_for_stage(stage),
        schedule.native_steps_for_stage(stage),
        schedule.native_conflict_graph_for_stage(stage),
    )
}

fn stage_tick_contexts(
    stage: SystemStage,
    snapshot: WorldTimeSnapshot,
    world_generation: u64,
    fixed_step: Option<&WorldFixedStep>,
    fixed_elapsed: std::time::Duration,
) -> Result<SceneStageTickContexts, CoreError> {
    let (simulation_tick, fixed_delta, fixed_elapsed) = match fixed_step {
        Some(step) => (Some(step.id()), step.timestep(), step.elapsed()),
        None => (None, snapshot.fixed_step_plan().timestep, fixed_elapsed),
    };
    let virtual_clock_domain = snapshot
        .clock_domain_stamp(crate::core::framework::time::ClockDomainId::WorldVirtual)
        .ok_or_else(|| {
            CoreError::Initialization(
                "WorldDriver clock context".to_string(),
                "WorldTimeSnapshot is missing its WorldVirtual clock stamp".to_string(),
            )
        })?;
    let fixed_clock_domain = snapshot
        .clock_domain_stamp(crate::core::framework::time::ClockDomainId::WorldFixed)
        .ok_or_else(|| {
            CoreError::Initialization(
                "WorldDriver clock context".to_string(),
                "WorldTimeSnapshot is missing its WorldFixed clock stamp".to_string(),
            )
        })?;
    Ok(SceneStageTickContexts::new(
        SystemTickContext::new(
            stage,
            virtual_clock_domain,
            snapshot.outer_frame_index(),
            None,
            snapshot.virtual_delta(),
            snapshot.virtual_elapsed(),
            world_generation,
        ),
        SystemTickContext::new(
            stage,
            snapshot.real_clock_domain_stamp(),
            snapshot.outer_frame_index(),
            None,
            snapshot.raw_real_delta(),
            snapshot.real_elapsed(),
            world_generation,
        ),
        SystemTickContext::new(
            stage,
            fixed_clock_domain,
            snapshot.outer_frame_index(),
            simulation_tick,
            fixed_delta,
            fixed_elapsed,
            world_generation,
        ),
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    use super::*;
    use crate::scene::{WorldRuntimeExtensionError, WorldRuntimeExtensionRegistration};

    #[test]
    fn world_runtime_extension_callbacks_apply_from_a_short_lock_snapshot() {
        let source = include_str!("world_driver.rs");
        let normalized = source
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        let snapshot = ["fnruntime_extension_plan_", "snapshot"].concat();
        let direct_apply = [
            "lock_poison_recovered(&self.runtime_extensions)",
            ".apply_to_world",
        ]
        .concat();

        assert!(normalized.contains(&snapshot));
        assert!(!normalized.contains(&direct_apply));
    }

    #[test]
    fn world_runtime_extension_snapshot_survives_a_later_plan_publication() {
        let driver = WorldDriver::default();
        driver
            .install_world_runtime_extension_plan(extension_plan("before"))
            .expect("initial extension plan");
        let before = driver.runtime_extension_plan_snapshot();

        driver
            .install_world_runtime_extension_plan(extension_plan("after"))
            .expect("replacement extension plan");
        let after = driver.runtime_extension_plan_snapshot();

        assert_eq!(before.registration_count(), 1);
        assert_eq!(after.registration_count(), 2);
        assert!(!Arc::ptr_eq(&before, &after));
    }

    #[test]
    fn world_runtime_extension_callback_can_publish_a_new_generation() {
        let driver = Arc::new(WorldDriver::default());
        let reentrant_driver = Arc::clone(&driver);
        driver
            .install_world_runtime_extension_plan(
                WorldRuntimeExtensionPlan::from_registrations([
                    WorldRuntimeExtensionRegistration::new("reentrant", move |_| {
                        let guard = reentrant_driver.runtime_extensions.try_lock().map_err(
                            |_| {
                                WorldRuntimeExtensionError::new(
                                    "world extension callback ran while the driver lock was held",
                                )
                            },
                        )?;
                        drop(guard);
                        reentrant_driver
                            .install_world_runtime_extension_plan(extension_plan("during.apply"))
                            .map_err(|error| WorldRuntimeExtensionError::new(error.to_string()))
                    }),
                ])
                .expect("reentrant extension plan"),
            )
            .expect("initial extension plan");

        let mut world = World::new();
        driver
            .apply_world_runtime_extensions(&mut world)
            .expect("extension callback can publish a successor generation");

        assert_eq!(
            driver
                .runtime_extension_plan_snapshot()
                .registration_count(),
            2
        );
    }

    #[test]
    fn world_runtime_extension_callbacks_overlap_across_independent_worlds() {
        const WORLD_COUNT: usize = 4;

        let driver = Arc::new(WorldDriver::default());
        let callbacks_in_flight = Arc::new(AtomicUsize::new(0));
        let peak_callbacks_in_flight = Arc::new(AtomicUsize::new(0));
        let callbacks_in_flight_for_registration = Arc::clone(&callbacks_in_flight);
        let peak_callbacks_in_flight_for_registration = Arc::clone(&peak_callbacks_in_flight);
        driver
            .install_world_runtime_extension_plan(
                WorldRuntimeExtensionPlan::from_registrations([
                    WorldRuntimeExtensionRegistration::new("concurrent", move |_| {
                        let in_flight =
                            callbacks_in_flight_for_registration.fetch_add(1, Ordering::SeqCst) + 1;
                        peak_callbacks_in_flight_for_registration
                            .fetch_max(in_flight, Ordering::SeqCst);
                        thread::sleep(Duration::from_millis(20));
                        callbacks_in_flight_for_registration.fetch_sub(1, Ordering::SeqCst);
                        Ok(())
                    }),
                ])
                .expect("concurrent extension plan"),
            )
            .expect("initial extension plan");

        let start = Arc::new(Barrier::new(WORLD_COUNT));
        let workers = (0..WORLD_COUNT)
            .map(|_| {
                let driver = Arc::clone(&driver);
                let start = Arc::clone(&start);
                thread::spawn(move || {
                    let mut world = World::new();
                    start.wait();
                    driver
                        .apply_world_runtime_extensions(&mut world)
                        .expect("world extension callback applies");
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            worker.join().expect("world extension worker completes");
        }

        assert!(
            peak_callbacks_in_flight.load(Ordering::SeqCst) > 1,
            "independent Worlds must not serialize callbacks behind the driver lock"
        );
    }

    fn extension_plan(key: &str) -> WorldRuntimeExtensionPlan {
        WorldRuntimeExtensionPlan::from_registrations([WorldRuntimeExtensionRegistration::new(
            key,
            |_| Ok(()),
        )])
        .expect("unique extension plan")
    }
}

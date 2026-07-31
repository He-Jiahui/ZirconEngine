use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::diagnostics::RuntimeDevtoolsSceneHookSnapshot;
use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle, RuntimeTimeAdvance};
use crate::scene::ecs::{SceneScheduleRunner, SystemStage};
use crate::scene::runtime_hook::{SceneRuntimeHookSet, SceneRuntimeHookStagePlan};
use crate::scene::{LevelSystem, SceneRuntimeHookRegistration, World, WorldRuntimeExtensionPlan};

#[derive(Debug, Default)]
pub struct WorldDriver {
    hooks: Mutex<SceneRuntimeHookSet>,
    runtime_extensions: Mutex<Arc<WorldRuntimeExtensionPlan>>,
}

impl WorldDriver {
    pub fn install_scene_runtime_hooks(
        &self,
        core: &CoreHandle,
        registrations: impl IntoIterator<Item = SceneRuntimeHookRegistration>,
    ) -> Result<(), CoreError> {
        let mut hooks = lock_poison_recovered(&self.hooks);
        let candidate = hooks.try_merge(registrations).map_err(|id| {
            CoreError::Initialization(
                "WorldDriver".to_string(),
                format!("duplicate scene runtime hook `{id}`"),
            )
        })?;
        core.replace_devtools_scene_hook_snapshots(
            candidate
                .ordered()
                .iter()
                .map(|hook| {
                    let descriptor = hook.descriptor();
                    RuntimeDevtoolsSceneHookSnapshot {
                        id: descriptor.id.clone(),
                        plugin_id: descriptor.plugin_id.clone(),
                        stage: format!("{:?}", descriptor.stage),
                        order: descriptor.order,
                    }
                })
                .collect(),
        );
        *hooks = candidate;
        Ok(())
    }

    pub fn scene_runtime_hooks_for_stage(
        &self,
        stage: SystemStage,
    ) -> Vec<SceneRuntimeHookRegistration> {
        lock_poison_recovered(&self.hooks)
            .hooks_for_stage(stage)
            .to_vec()
    }

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

    fn scene_runtime_hook_stage_plan_snapshot(&self) -> Arc<SceneRuntimeHookStagePlan> {
        lock_poison_recovered(&self.hooks).stage_plan()
    }

    pub fn tick_level(
        &self,
        core: &CoreHandle,
        level: &LevelSystem,
        advance: RuntimeTimeAdvance,
    ) -> Result<(), CoreError> {
        let delta_seconds = duration_to_real_seconds(advance.real_delta());
        let fixed_step_plan = advance.fixed_step_plan();
        let fixed_delta_seconds = duration_to_real_seconds(fixed_step_plan.timestep);
        let schedule = level.with_world(|world| world.schedule().stage_plan());
        let hooks = self.scene_runtime_hook_stage_plan_snapshot();
        level.with_world_mut(|world| world.reset_ecs_frame_performance_diagnostics());
        for stage in schedule.stages() {
            if *stage == SystemStage::FixedFirst {
                for _ in 0..fixed_step_plan.step_count {
                    for fixed_stage in SystemStage::FIXED_LOOP {
                        run_stage(
                            core,
                            level,
                            fixed_stage,
                            fixed_delta_seconds,
                            &schedule,
                            hooks.hooks_for_stage(fixed_stage),
                        )?;
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
                delta_seconds,
                &schedule,
                hooks.hooks_for_stage(*stage),
            )?;
        }

        level.with_world(|world| {
            world
                .ecs_frame_performance_diagnostics()
                .publish(core, core.real_time().frame_index());
        });

        Ok(())
    }
}

fn lock_poison_recovered<T>(lock: &Mutex<T>) -> MutexGuard<'_, T> {
    lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn run_stage(
    core: &CoreHandle,
    level: &LevelSystem,
    stage: SystemStage,
    delta_seconds: Real,
    schedule: &crate::scene::ecs::SceneScheduleStagePlan,
    hooks: &[SceneRuntimeHookRegistration],
) -> Result<(), CoreError> {
    SceneScheduleRunner::run_stage(
        core,
        level,
        stage,
        delta_seconds,
        schedule.internal_systems_for_stage(stage),
        schedule.native_steps_for_stage(stage),
        schedule.native_conflict_graph_for_stage(stage),
        hooks,
    )
}

fn duration_to_real_seconds(duration: std::time::Duration) -> Real {
    let seconds = duration.as_secs_f64() as Real;
    if seconds.is_finite() {
        seconds.max(0.0)
    } else {
        0.0
    }
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
                        let guard =
                            reentrant_driver
                                .runtime_extensions
                                .try_lock()
                                .map_err(|_| {
                                    WorldRuntimeExtensionError::new(
                                "world extension callback ran while the driver lock was held",
                            )
                                })?;
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

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
    runtime_extensions: Mutex<WorldRuntimeExtensionPlan>,
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
        let candidate = plan.try_merge(contribution).map_err(|error| {
            CoreError::Initialization("WorldDriver".to_string(), error.to_string())
        })?;
        *plan = candidate;
        Ok(())
    }

    pub fn apply_world_runtime_extensions(&self, world: &mut World) -> Result<(), CoreError> {
        lock_poison_recovered(&self.runtime_extensions)
            .apply_to_world(world)
            .map_err(|error| {
                CoreError::Initialization("WorldDriver".to_string(), error.to_string())
            })
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

use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle, RuntimeTimeAdvance};
use crate::plugin::SceneRuntimeHookRegistration;
use crate::scene::ecs::{SceneScheduleRunner, SystemStage};
use crate::scene::LevelSystem;

#[derive(Debug, Default)]
pub struct WorldDriver;

impl WorldDriver {
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
        let hooks = core.scene_runtime_hook_stage_plan_snapshot();
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

        Ok(())
    }
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

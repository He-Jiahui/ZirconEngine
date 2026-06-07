use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle};
use crate::scene::ecs::SceneScheduleRunner;
use crate::scene::LevelSystem;

#[derive(Debug, Default)]
pub struct WorldDriver;

impl WorldDriver {
    pub fn tick_level(
        &self,
        core: &CoreHandle,
        level: &LevelSystem,
        delta_seconds: Real,
    ) -> Result<(), CoreError> {
        let schedule = level.with_world(|world| world.schedule().stage_plan());
        let hooks = core.scene_runtime_hook_stage_plan_snapshot();
        for stage in schedule.stages() {
            SceneScheduleRunner::run_stage(
                core,
                level,
                *stage,
                delta_seconds,
                schedule.internal_systems_for_stage(*stage),
                schedule.native_steps_for_stage(*stage),
                hooks.hooks_for_stage(*stage),
            )?;
        }

        Ok(())
    }
}

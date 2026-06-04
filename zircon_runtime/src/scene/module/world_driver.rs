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
        let (stages, systems) = level.with_world(|world| {
            (
                world.schedule().stages.clone(),
                world.schedule().systems().to_vec(),
            )
        });
        let hooks = core.scene_runtime_hooks_snapshot();
        for stage in stages {
            SceneScheduleRunner::run_stage(core, level, stage, delta_seconds, &systems, &hooks)?;
        }

        Ok(())
    }
}

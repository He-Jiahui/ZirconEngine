use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle};
use crate::scene::ecs::SceneScheduleRunner;
use crate::scene::{LevelSystem, SystemStage};

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
        for stage in stages {
            let hooks = core.scene_runtime_hooks_for_stage(stage);
            SceneScheduleRunner::run_stage(
                core,
                level,
                stage,
                delta_seconds,
                systems_for_stage(&systems, stage),
                hooks,
            )?;
        }

        Ok(())
    }
}

fn systems_for_stage(
    systems: &[crate::scene::ecs::SceneSystemDescriptor],
    stage: SystemStage,
) -> Vec<crate::scene::ecs::SceneSystemDescriptor> {
    systems
        .iter()
        .filter(|system| system.stage == stage)
        .cloned()
        .collect()
}

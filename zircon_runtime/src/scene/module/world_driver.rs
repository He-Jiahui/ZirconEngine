use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle};
use crate::plugin::SceneRuntimeHookContext;
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
        let stages = level.with_world(|world| world.schedule().stages.clone());
        for stage in stages {
            for hook in core.scene_runtime_hooks_for_stage(stage) {
                hook.run(SceneRuntimeHookContext::new(core, level, delta_seconds))?;
            }
        }

        Ok(())
    }
}

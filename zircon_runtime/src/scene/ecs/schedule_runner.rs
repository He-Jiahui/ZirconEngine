use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle};
use crate::plugin::{SceneRuntimeHookContext, SceneRuntimeHookRegistration};
use crate::scene::ecs::{
    InternalSceneSystem, SceneSystemDescriptor, ScheduledSceneStep, SystemStage,
};
use crate::scene::LevelSystem;

pub(crate) struct SceneScheduleRunner;

impl SceneScheduleRunner {
    pub(crate) fn run_stage(
        core: &CoreHandle,
        level: &LevelSystem,
        stage: SystemStage,
        delta_seconds: Real,
        internal_systems: Vec<SceneSystemDescriptor>,
        hooks: Vec<SceneRuntimeHookRegistration>,
    ) -> Result<(), CoreError> {
        level.with_world_mut(|world| world.set_scene_system_flush_deferred(true));
        let native_steps =
            level.with_world(|world| world.scheduled_native_system_steps_for_stage(stage));
        let steps =
            ScheduledSceneStep::sorted_for_stage(stage, internal_systems, native_steps, hooks);

        let result = (|| {
            for step in steps {
                match step {
                    ScheduledSceneStep::Internal(system) => {
                        level.with_world_mut(|world| {
                            world.run_internal_scene_system(system.system())
                        });
                        if system.system() != InternalSceneSystem::ApplyDeferred {
                            level.with_world_mut(|world| world.apply_deferred());
                        }
                    }
                    ScheduledSceneStep::Native { id, .. } => {
                        level.with_world_mut(|world| world.run_native_scene_system(&id));
                    }
                    ScheduledSceneStep::ApplyDeferred { .. } => {
                        level.with_world_mut(|world| world.apply_deferred());
                    }
                    ScheduledSceneStep::Hook(hook) => {
                        hook.run(SceneRuntimeHookContext::new(core, level, delta_seconds))?;
                        level.with_world_mut(|world| world.apply_deferred());
                    }
                }
            }

            Ok(())
        })();
        level.with_world_mut(|world| {
            world.set_scene_system_flush_deferred(false);
            if result.is_ok() {
                world.run_internal_scene_systems_for_stage(stage);
            }
        });
        result
    }
}

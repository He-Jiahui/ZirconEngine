use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle};
use crate::scene::ecs::{
    InternalSceneSystem, SceneSystemDescriptor, ScheduledSceneStep, ScheduledSceneStepRef,
    SystemStage,
};
use crate::scene::LevelSystem;
use crate::scene::{SceneRuntimeHookContext, SceneRuntimeHookRegistration};

pub(crate) struct SceneScheduleRunner;

impl SceneScheduleRunner {
    pub(crate) fn run_stage(
        core: &CoreHandle,
        level: &LevelSystem,
        stage: SystemStage,
        delta_seconds: Real,
        internal_systems: &[SceneSystemDescriptor],
        native_steps: &[ScheduledSceneStep],
        hooks: &[SceneRuntimeHookRegistration],
    ) -> Result<(), CoreError> {
        crate::profile_dynamic_scope!(
            "runtime",
            "frame",
            format!("runtime_frame_schedule_stage.{stage:?}"),
        );

        level.with_world_mut(|world| world.set_scene_system_flush_deferred(true));

        let result = (|| {
            for step in ScheduledSceneStep::iter_sorted_for_stage(
                stage,
                internal_systems,
                native_steps,
                hooks,
            ) {
                match step {
                    ScheduledSceneStepRef::Internal(system) => {
                        level.with_world_mut(|world| {
                            world.run_internal_scene_system(system.system())
                        });
                        if !matches!(
                            system.system(),
                            InternalSceneSystem::ApplyDeferred | InternalSceneSystem::UpdateEvents
                        ) {
                            level.with_world_mut(|world| world.apply_deferred());
                        }
                    }
                    ScheduledSceneStepRef::Native { id, .. } => {
                        level.with_world_mut(|world| world.run_native_scene_system(id));
                    }
                    ScheduledSceneStepRef::Runtime { id, .. } => {
                        level.run_runtime_scene_system(core, id, delta_seconds)?;
                        level.with_world_mut(|world| world.apply_deferred());
                    }
                    ScheduledSceneStepRef::ApplyDeferred { .. } => {
                        level.with_world_mut(|world| world.apply_deferred());
                    }
                    ScheduledSceneStepRef::Hook(hook) => {
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
                world.flush_pending_scene_systems_for_stage(stage);
            }
        });
        result
    }
}

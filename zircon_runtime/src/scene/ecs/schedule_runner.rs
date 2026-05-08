use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle};
use crate::plugin::{SceneRuntimeHookContext, SceneRuntimeHookRegistration};
use crate::scene::ecs::{SceneSystemDescriptor, SystemStage};
use crate::scene::LevelSystem;

pub(crate) struct SceneScheduleRunner;

enum SceneScheduleStep {
    Internal(SceneSystemDescriptor),
    Hook(SceneRuntimeHookRegistration),
}

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
        let mut steps = internal_systems
            .into_iter()
            .filter(|system| system.stage == stage)
            .map(SceneScheduleStep::Internal)
            .chain(hooks.into_iter().map(SceneScheduleStep::Hook))
            .collect::<Vec<_>>();
        steps.sort_by(|left, right| {
            left.order()
                .cmp(&right.order())
                .then(left.id().cmp(right.id()))
        });

        let result = (|| {
            for step in steps {
                match step {
                    SceneScheduleStep::Internal(system) => {
                        level.with_world_mut(|world| {
                            world.run_internal_scene_system(system.system())
                        });
                    }
                    SceneScheduleStep::Hook(hook) => {
                        hook.run(SceneRuntimeHookContext::new(core, level, delta_seconds))?;
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

impl SceneScheduleStep {
    fn order(&self) -> i32 {
        match self {
            Self::Internal(system) => system.order,
            Self::Hook(hook) => hook.descriptor().order,
        }
    }

    fn id(&self) -> &str {
        match self {
            Self::Internal(system) => system.id.as_str(),
            Self::Hook(hook) => hook.descriptor().id.as_str(),
        }
    }
}

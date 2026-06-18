use crate::core::CoreError;
use crate::plugin::{
    SceneRuntimeHook, SceneRuntimeHookContext, SceneRuntimeHookDescriptor,
    SceneRuntimeHookRegistration,
};
use crate::scene::SystemStage;

mod diagnostics;
mod events;
mod graph;
mod node_pose;
mod pending;
mod pose;
mod scan;
mod sequences;
mod state_machine;
mod tick;

#[derive(Clone, Debug, Default)]
pub struct AnimationSceneRuntimeHook;

pub fn scene_hook_registration() -> SceneRuntimeHookRegistration {
    SceneRuntimeHookRegistration::new(
        SceneRuntimeHookDescriptor::new(
            "animation.scene.post_update",
            crate::animation::PLUGIN_ID,
            SystemStage::PostUpdate,
        ),
        AnimationSceneRuntimeHook,
    )
}

impl SceneRuntimeHook for AnimationSceneRuntimeHook {
    fn run(&self, context: SceneRuntimeHookContext<'_>) -> Result<(), CoreError> {
        tick::tick_animation_world(context.core, context.level, context.delta_seconds);
        Ok(())
    }
}

use zircon_runtime::core::CoreError;
use zircon_runtime::plugin::PluginEventManifest;
use zircon_runtime::plugin::RuntimeExtensionRegistryError;
use zircon_runtime::scene::ecs::{RuntimeSceneSystemContext, SystemRef};
use zircon_runtime::scene::SystemStage;

#[derive(Clone, Debug, Default)]
pub struct AnimationRuntimeSystem;

pub const ANIMATION_SYSTEM_SET: &str = "animation.evaluation";
pub const ANIMATION_EVALUATE_SYSTEM: &str = "animation.evaluate";
pub const ANIMATION_EVALUATION_DIAGNOSTIC_EVENT: &str = "animation.events.evaluation_diagnostic";
pub const ANIMATION_EVALUATION_DIAGNOSTIC_SCHEMA: &str = "animation.evaluation_diagnostic.v1";
pub const ANIMATION_CLIP_EVENT: &str = "animation.events.clip";
pub const ANIMATION_CLIP_EVENT_SCHEMA: &str = "animation.clip_event.v1";
pub const ANIMATION_IK_DIAGNOSTIC_EVENT: &str = "animation.events.ik_diagnostic";
pub const ANIMATION_IK_DIAGNOSTIC_SCHEMA: &str = "animation.ik_diagnostic.v1";
pub const ANIMATION_LAYER_DIAGNOSTIC_EVENT: &str = "animation.events.layer_diagnostic";
pub const ANIMATION_LAYER_DIAGNOSTIC_SCHEMA: &str = "animation.layer_diagnostic.v1";

pub fn register_runtime_system(
    module: &mut zircon_plugin_sdk::RuntimePluginModuleRegistration<'_>,
) -> Result<(), RuntimeExtensionRegistryError> {
    module.event::<crate::AnimationClipEvent>(PluginEventManifest {
        id: ANIMATION_CLIP_EVENT.to_string(),
        display_name: "Animation Clip Event".to_string(),
        payload_schema: ANIMATION_CLIP_EVENT_SCHEMA.to_string(),
    })?;
    module.event::<crate::AnimationEvaluationDiagnostic>(PluginEventManifest {
        id: ANIMATION_EVALUATION_DIAGNOSTIC_EVENT.to_string(),
        display_name: "Animation Evaluation Diagnostic".to_string(),
        payload_schema: ANIMATION_EVALUATION_DIAGNOSTIC_SCHEMA.to_string(),
    })?;
    module.event::<crate::AnimationIkDiagnostic>(PluginEventManifest {
        id: ANIMATION_IK_DIAGNOSTIC_EVENT.to_string(),
        display_name: "Animation IK Diagnostic".to_string(),
        payload_schema: ANIMATION_IK_DIAGNOSTIC_SCHEMA.to_string(),
    })?;
    module.event::<crate::AnimationStateMachineLayerDiagnostic>(PluginEventManifest {
        id: ANIMATION_LAYER_DIAGNOSTIC_EVENT.to_string(),
        display_name: "Animation State Machine Layer Diagnostic".to_string(),
        payload_schema: ANIMATION_LAYER_DIAGNOSTIC_SCHEMA.to_string(),
    })?;
    module.resource(crate::AnimationEvaluationPipeline::default)?;
    module
        .runtime_scene_system(
            ANIMATION_EVALUATE_SYSTEM,
            SystemStage::PostUpdate,
            run_animation_runtime_system,
        )
        .in_set(ANIMATION_SYSTEM_SET)
        .after(SystemRef::System(
            "zircon.scene.world_transform".to_string(),
        ))
        .register()
}

fn run_animation_runtime_system(context: RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> {
    crate::evaluation::tick_animation_world(context.core, context.level, context.delta_seconds);
    Ok(())
}

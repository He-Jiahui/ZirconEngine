//! 动画评估系统在场景变换更新之后的 PostUpdate 阶段消费播放器和资源快照。
//! 每帧由 pipeline 写回场景局部变换、姿态快照和事件，调用者需保证 LevelSystem 仍处于同一替换纪元。
use zircon_runtime::core::CoreError;
#[cfg(feature = "native-host")]
use zircon_runtime::plugin::PluginEventManifest;
#[cfg(feature = "native-host")]
use zircon_runtime::plugin::RuntimeExtensionRegistryError;
use zircon_runtime::scene::ecs::{RuntimeSceneSystemContext, SystemRef};
use zircon_runtime::scene::SystemStage;

#[derive(Clone, Debug, Default)]
pub struct AnimationRuntimeSystem;

pub const ANIMATION_SYSTEM_SET: &str = "animation.main";
pub const ANIMATION_EVALUATE_SYSTEM: &str = "animation.evaluate";
pub const ANIMATION_EVALUATION_DIAGNOSTIC_EVENT: &str = "animation.events.evaluation_diagnostic";
pub const ANIMATION_EVALUATION_DIAGNOSTIC_SCHEMA: &str = "animation.evaluation_diagnostic.v1";
pub const ANIMATION_CLIP_EVENT: &str = "animation.events.clip";
pub const ANIMATION_CLIP_EVENT_SCHEMA: &str = "animation.clip_event.v1";
pub const ANIMATION_LAYER_DIAGNOSTIC_EVENT: &str = "animation.events.layer_diagnostic";
pub const ANIMATION_LAYER_DIAGNOSTIC_SCHEMA: &str = "animation.layer_diagnostic.v1";

/// 注册管线资源、评估系统和场景事件类型；应由插件注册生命周期调用一次。
#[cfg(feature = "native-host")]
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
    module.event::<crate::AnimationStateMachineLayerDiagnostic>(PluginEventManifest {
        id: ANIMATION_LAYER_DIAGNOSTIC_EVENT.to_string(),
        display_name: "Animation State Machine Layer Diagnostic".to_string(),
        payload_schema: ANIMATION_LAYER_DIAGNOSTIC_SCHEMA.to_string(),
    })?;
    module.resource(crate::AnimationEvaluationPipeline::default)?;
    module
        .runtime_scene_system(ANIMATION_EVALUATE_SYSTEM, SystemStage::PostUpdate, || {
            run_animation_runtime_system
        })
        .in_set(ANIMATION_SYSTEM_SET)
        .after(SystemRef::System(
            "zircon.scene.world_transform".to_string(),
        ))
        .register()
}

fn run_animation_runtime_system(context: RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> {
    crate::evaluation::tick_animation_world(
        context.core,
        context.level,
        context.tick().delta_seconds(),
    );
    Ok(())
}

/// Uses the same evaluator and scheduling stage as the desktop registration.
pub fn portable_runtime_extension_plan() -> Result<
    zircon_runtime::scene::WorldRuntimeExtensionPlan,
    zircon_runtime::scene::WorldRuntimeExtensionError,
> {
    use zircon_runtime::scene::ecs::{FunctionRuntimeSceneSystem, SceneSystemMetadata};
    use zircon_runtime::scene::{
        WorldRuntimeExtensionError, WorldRuntimeExtensionPlan, WorldRuntimeExtensionRegistration,
    };
    WorldRuntimeExtensionPlan::from_registrations([WorldRuntimeExtensionRegistration::new(
        ANIMATION_EVALUATE_SYSTEM,
        |world| {
            world.register_event::<crate::AnimationClipEvent>();
            world.register_event::<crate::AnimationEvaluationDiagnostic>();
            world.register_event::<crate::AnimationStateMachineLayerDiagnostic>();
            world.insert_resource(crate::AnimationEvaluationPipeline::default());
            world
                .register_boxed_runtime_scene_system(Box::new(FunctionRuntimeSceneSystem::new(
                    SceneSystemMetadata::new(ANIMATION_EVALUATE_SYSTEM, SystemStage::PostUpdate, 0),
                    run_animation_runtime_system,
                )))
                .map_err(|error| {
                    WorldRuntimeExtensionError::registration_failed(
                        ANIMATION_EVALUATE_SYSTEM,
                        error,
                    )
                })
        },
    )])
}

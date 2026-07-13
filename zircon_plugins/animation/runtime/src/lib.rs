pub const PLUGIN_ID: &str = "animation";
mod capability;
mod evaluation;
mod gpu_skinning;
mod ik;
mod mask;
mod plugin;
mod runtime_system;
mod state_machine;

pub use capability::{
    ANIMATION_RUNTIME_CAPABILITY, ANIMATION_TIMELINE_EVENT_TRACK_CAPABILITY, RUNTIME_CAPABILITIES,
};
pub use evaluation::{
    AnimationAssetRevision, AnimationChannelDataRole, AnimationClipCompileError,
    AnimationClipEvaluator, AnimationClipEvaluatorStats, AnimationEvaluationDiagnostic,
    AnimationEvaluationError, AnimationEvaluationPipeline, AnimationGraphCompileError,
    AnimationStateMachineLayerDiagnostic, AnimationStateMachineLayerError,
    AnimationTransformChannel, CompiledAnimationClip, CompiledAnimationGraph,
    CompiledAnimationGraphEvaluation, CompiledClipTrack, CompiledGraphClipInstance, PoseBlendError,
    PoseBuffer, PoseBufferError, PoseLayer, PoseLayerBlendMode, PosePool, SkeletonTargetTable,
};
pub use gpu_skinning::{
    AnimationGpuSkinningDecision, SkinningPalette, SkinningPaletteDoubleBuffer,
    SkinningPaletteError, MAX_SKIN_JOINTS,
};
pub use ik::{
    AnimationIkDiagnostic, AnimationIkError, AnimationIkExecutionError, LookAtJob, TwoBoneIkJob,
    TwoBoneIkSolution,
};
pub use mask::{AvatarMaskAsset, AvatarMaskError, AvatarMaskRule, MaskWeights};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, AnimationRuntimePlugin, ANIMATION_DIST_CRATE_NAME,
    ANIMATION_DIST_RUNTIME_ENTRY, PLUGIN_RUNTIME_MODULE_NAME,
};
pub use runtime_system::{
    register_runtime_system, AnimationRuntimeSystem, ANIMATION_CLIP_EVENT,
    ANIMATION_CLIP_EVENT_SCHEMA, ANIMATION_EVALUATE_SYSTEM, ANIMATION_EVALUATION_DIAGNOSTIC_EVENT,
    ANIMATION_EVALUATION_DIAGNOSTIC_SCHEMA, ANIMATION_IK_DIAGNOSTIC_EVENT,
    ANIMATION_IK_DIAGNOSTIC_SCHEMA, ANIMATION_LAYER_DIAGNOSTIC_EVENT,
    ANIMATION_LAYER_DIAGNOSTIC_SCHEMA, ANIMATION_SYSTEM_SET,
};
pub use state_machine::{
    AnimationStateMachineCompileError, BlendSpace1D, BlendSpace2D, BlendSpaceCompileError,
    BlendSpacePoint1D, BlendSpacePoint2D, BlendSpaceWeights2, BlendSpaceWeights3,
    CompiledAnimationStateMachine, CompiledConditionExpression, CompiledStateMachineEvaluation,
    CompiledStateMachineLayer, CompiledStateMachineLayers, ConditionExpression,
    ConditionExpressionCompileError, InterruptionPolicy, StateMachineLayerCompileError,
    TransitionDesc, TransitionRequest, TransitionRuntime, TransitionState, TransitionWeights,
};
pub use zircon_runtime::animation::{
    apply_sequence_to_world, module_descriptor, sample_clip_events, AnimationClipEvent,
    AnimationDriver, AnimationModule, DefaultAnimationManager, ANIMATION_DRIVER_NAME,
    ANIMATION_MODULE_NAME, ANIMATION_PLAYBACK_CONFIG_KEY, DEFAULT_ANIMATION_MANAGER_NAME,
};
pub use zircon_runtime::core::framework::animation::AnimationSequenceApplyReport;
pub use zircon_runtime::core::manager::ANIMATION_MANAGER_NAME;

#[cfg(test)]
mod tests;

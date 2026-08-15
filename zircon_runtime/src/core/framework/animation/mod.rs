//! Animation framework contracts for sequence, graph, state-machine, parameter, and pose evaluation.

pub mod asset;
mod avatar_mask;
mod clip_event_sampling;
mod error;
mod event;
mod gpu_skinning;
mod graph_blend_mode;
mod graph_clip_instance;
mod graph_evaluation;
mod ik_command;
mod ik_command_error;
mod manager;
mod parameter_map;
mod parameter_value;
mod playback_settings;
mod pose_bone;
mod pose_output;
mod pose_source;
mod runtime_status;
mod sequence_apply_report;
mod state_machine_evaluation;
mod target_id;
mod tick;
mod timeline;
mod track_path;
mod track_path_error;

pub use asset::{
    AnimationAssetError, AnimationAssetResult, AnimationBlendSpace1DAsset,
    AnimationBlendSpace1DSampleAsset, AnimationBlendSpace2DAsset, AnimationBlendSpace2DSampleAsset,
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationConditionOperatorAsset,
    AnimationEventTrackAsset, AnimationGraphAsset, AnimationGraphNodeAsset,
    AnimationGraphParameterAsset, AnimationInterpolationAsset, AnimationSequenceAsset,
    AnimationSequenceBindingAsset, AnimationSequenceTrackAsset, AnimationSkeletonAsset,
    AnimationSkeletonBoneAsset, AnimationStateAsset, AnimationStateKindAsset,
    AnimationStateMachineAsset, AnimationStateMachineLayerAsset,
    AnimationStateMachineLayerBlendModeAsset, AnimationStateTransitionAsset,
    AnimationTransitionConditionAsset, AnimationTransitionInterruptionPolicyAsset,
};
pub use avatar_mask::AnimationAvatarMask;
pub use clip_event_sampling::{
    AnimationClipEvent, AnimationClipEventBatchAdmission, AnimationClipEventQueueAdmission,
    AnimationClipEventSampler, AnimationClipEventSamplingBatch, AnimationClipEventSamplingCursor,
    AnimationClipEventSamplingLimits, AnimationClipEventSamplingRange,
    AnimationClipEventSamplingRequest,
};
pub use error::{AnimationError, AnimationResult};
pub use event::AnimationEventRecord;
pub use gpu_skinning::{AnimationGpuSkinningReadiness, AnimationSkinningBackend};
pub use graph_blend_mode::AnimationGraphBlendMode;
pub use graph_clip_instance::AnimationGraphClipInstance;
pub use graph_evaluation::AnimationGraphEvaluation;
pub use ik_command::{AnimationIkCommand, AnimationLookAtCommand, AnimationTwoBoneIkCommand};
pub use ik_command_error::AnimationIkCommandError;
pub use manager::AnimationManager;
pub use parameter_map::AnimationParameterMap;
pub use parameter_value::AnimationParameterValue;
pub use playback_settings::AnimationPlaybackSettings;
pub use pose_bone::AnimationPoseBone;
pub use pose_output::AnimationPoseOutput;
pub use pose_source::AnimationPoseSource;
pub use runtime_status::{
    AnimationPlayerKind, AnimationPlayerRuntimeState, AnimationPlayerRuntimeStatus,
    AnimationRigRuntimeStatus, AnimationRuntimeStatus,
};
pub use sequence_apply_report::AnimationSequenceApplyReport;
pub use state_machine_evaluation::{
    AnimationStateMachineEvaluation, AnimationStateTransitionEvaluation,
};
pub use target_id::AnimationTargetId;
pub use tick::{AnimationTickReport, AnimationTickRequest};
pub use timeline::{
    AnimationTimelineClipDescriptor, AnimationTimelineDescriptor, AnimationTimelineEventDescriptor,
    AnimationTimelineTrackDescriptor, AnimationTimelineTrackKind,
};
pub use track_path::AnimationTrackPath;
pub use track_path_error::AnimationTrackPathError;

#[cfg(test)]
mod tests;

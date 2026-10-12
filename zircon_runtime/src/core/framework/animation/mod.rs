//! Animation framework contracts for sequence, graph, state-machine, parameter, and pose evaluation.

#[path = "asset/mod.rs"]
pub mod asset;
#[path = "avatar_mask.rs"]
mod avatar_mask;
#[path = "clip_event_sampling.rs"]
mod clip_event_sampling;
#[path = "compiler/mod.rs"]
pub mod compiler;
#[path = "error.rs"]
mod error;
#[path = "event.rs"]
mod event;
#[path = "gpu_skinning.rs"]
mod gpu_skinning;
#[path = "graph_blend_mode.rs"]
mod graph_blend_mode;
#[path = "graph_clip_instance.rs"]
mod graph_clip_instance;
#[path = "graph_evaluation.rs"]
mod graph_evaluation;
#[path = "manager.rs"]
mod manager;
#[path = "parameter_map.rs"]
mod parameter_map;
#[path = "parameter_set.rs"]
mod parameter_set;
#[path = "parameter_value.rs"]
mod parameter_value;
#[path = "playback_settings.rs"]
mod playback_settings;
#[path = "pose_bone.rs"]
mod pose_bone;
#[path = "pose_output.rs"]
mod pose_output;
#[path = "pose_snapshot.rs"]
mod pose_snapshot;
#[path = "pose_source.rs"]
mod pose_source;
#[path = "runtime_status.rs"]
mod runtime_status;
#[path = "sequence_apply_report.rs"]
mod sequence_apply_report;
#[path = "state_machine_evaluation.rs"]
mod state_machine_evaluation;
#[path = "target_id.rs"]
mod target_id;
#[path = "tick.rs"]
mod tick;
#[path = "timeline.rs"]
mod timeline;
#[path = "track_path.rs"]
mod track_path;
#[path = "track_path_error.rs"]
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
pub use manager::AnimationManager;
pub use parameter_map::AnimationParameterMap;
pub use parameter_set::{
    AnimationParameterContentFingerprint, AnimationParameterRevision, AnimationParameterSet,
};
pub use parameter_value::AnimationParameterValue;
pub use playback_settings::AnimationPlaybackSettings;
pub use pose_bone::AnimationPoseBone;
pub use pose_output::AnimationPoseOutput;
pub use pose_snapshot::{AnimationPoseHandle, AnimationPoseMap, AnimationPoseSnapshot};
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
#[path = "tests/cases.rs"]
mod tests;

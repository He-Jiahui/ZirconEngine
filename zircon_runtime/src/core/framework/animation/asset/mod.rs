//! Versioned animation resource schemas and their stable binary wire format.

mod binary;
mod channel;
mod clip;
mod error;
mod graph;
mod reference;
mod sequence;
mod skeleton;
mod state_kind;
mod state_machine;

pub use channel::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset,
};
pub use clip::{AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationEventTrackAsset};
pub use error::{AnimationAssetError, AnimationAssetResult};
pub use graph::{AnimationGraphAsset, AnimationGraphNodeAsset, AnimationGraphParameterAsset};
pub use sequence::{
    AnimationSequenceAsset, AnimationSequenceBindingAsset, AnimationSequenceTrackAsset,
};
pub use skeleton::{AnimationSkeletonAsset, AnimationSkeletonBoneAsset};
pub use state_kind::{
    AnimationBlendSpace1DAsset, AnimationBlendSpace1DSampleAsset, AnimationBlendSpace2DAsset,
    AnimationBlendSpace2DSampleAsset, AnimationStateKindAsset,
};
pub use state_machine::{
    AnimationConditionOperatorAsset, AnimationStateAsset, AnimationStateMachineAsset,
    AnimationStateMachineLayerAsset, AnimationStateMachineLayerBlendModeAsset,
    AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
    AnimationTransitionInterruptionPolicyAsset,
};

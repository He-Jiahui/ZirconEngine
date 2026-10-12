//! Versioned animation resource schemas and their stable binary wire format.

#[path = "binary.rs"]
mod binary;
#[path = "channel.rs"]
mod channel;
#[path = "clip.rs"]
mod clip;
#[path = "error.rs"]
mod error;
#[path = "graph.rs"]
mod graph;
#[path = "reference.rs"]
mod reference;
#[path = "sequence.rs"]
mod sequence;
#[path = "skeleton.rs"]
mod skeleton;
#[path = "state_kind.rs"]
mod state_kind;
#[path = "state_machine.rs"]
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

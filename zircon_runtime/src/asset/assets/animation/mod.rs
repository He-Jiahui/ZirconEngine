mod binary;
mod channel;
mod clip;
mod error;
mod graph;
mod reference;
mod sequence;
mod skeleton;
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
pub use state_machine::{
    AnimationConditionOperatorAsset, AnimationStateAsset, AnimationStateMachineAsset,
    AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
};

use std::collections::BTreeMap;

use crate::core::framework::animation::AnimationParameterValue;
use crate::core::math::Real;
use crate::core::resource::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, ResourceHandle,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationSkeletonComponent {
    pub skeleton: ResourceHandle<AnimationSkeletonMarker>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationPlayerComponent {
    pub clip: ResourceHandle<AnimationClipMarker>,
    pub playback_speed: Real,
    pub time_seconds: Real,
    pub weight: Real,
    pub looping: bool,
    pub playing: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationSequencePlayerComponent {
    pub sequence: ResourceHandle<AnimationSequenceMarker>,
    pub playback_speed: Real,
    pub time_seconds: Real,
    pub looping: bool,
    pub playing: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationGraphPlayerComponent {
    pub graph: ResourceHandle<AnimationGraphMarker>,
    pub parameters: BTreeMap<String, AnimationParameterValue>,
    pub playing: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationStateMachinePlayerComponent {
    pub state_machine: ResourceHandle<AnimationStateMachineMarker>,
    pub parameters: BTreeMap<String, AnimationParameterValue>,
    pub active_state: Option<String>,
    pub playing: bool,
}

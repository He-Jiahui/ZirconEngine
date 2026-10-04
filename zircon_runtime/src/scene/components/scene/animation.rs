use crate::core::framework::animation::AnimationParameterSet;
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
/// 资源与播放参数的场景配置；LevelSystem 在固定帧中消费它并把姿态作为帧快照发布给渲染提取。
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
    pub parameters: AnimationParameterSet,
    pub playing: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationStateMachinePlayerComponent {
    pub state_machine: ResourceHandle<AnimationStateMachineMarker>,
    pub parameters: AnimationParameterSet,
    pub active_state: Option<String>,
    pub playing: bool,
}

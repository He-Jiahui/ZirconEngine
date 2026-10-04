//! 场景动画组件只保存对骨架、序列或图的身份与播放状态；加载侧按引用解析资源，运行时播放器拥有实际采样与状态推进。

use crate::asset::AssetReference;
use crate::core::framework::animation::AnimationParameterSet;
use crate::core::math::Real;
use serde::{Deserialize, Serialize};

use super::defaults::{default_animation_weight, default_playback_speed, default_true};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAnimationSkeletonAsset {
    pub skeleton: AssetReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAnimationPlayerAsset {
    pub clip: AssetReference,
    #[serde(default = "default_playback_speed")]
    pub playback_speed: Real,
    #[serde(default)]
    pub time_seconds: Real,
    #[serde(default = "default_animation_weight")]
    pub weight: Real,
    #[serde(default = "default_true")]
    pub looping: bool,
    #[serde(default = "default_true")]
    pub playing: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAnimationSequencePlayerAsset {
    pub sequence: AssetReference,
    #[serde(default = "default_playback_speed")]
    pub playback_speed: Real,
    #[serde(default)]
    pub time_seconds: Real,
    #[serde(default = "default_true")]
    pub looping: bool,
    #[serde(default = "default_true")]
    pub playing: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAnimationGraphPlayerAsset {
    pub graph: AssetReference,
    #[serde(default)]
    pub parameters: AnimationParameterSet,
    #[serde(default = "default_true")]
    pub playing: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAnimationStateMachinePlayerAsset {
    pub state_machine: AssetReference,
    #[serde(default)]
    pub parameters: AnimationParameterSet,
    #[serde(default)]
    pub active_state: Option<String>,
    #[serde(default = "default_true")]
    pub playing: bool,
}

use serde::{Deserialize, Serialize};

use crate::core::math::Real;
use crate::core::resource::AssetReference;

use super::AnimationParameterMap;

/// 一次状态机求值的选择结果；调用方根据状态和外部图引用继续采样，
/// `transition` 保留跨帧混合所需的起止状态与时长。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AnimationStateMachineEvaluation {
    pub parameters: AnimationParameterMap,
    pub active_state: Option<String>,
    pub transitioned: bool,
    pub graph: Option<AssetReference>,
    pub transition: Option<AnimationStateTransitionEvaluation>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationStateTransitionEvaluation {
    pub from_state: String,
    pub to_state: String,
    pub duration_seconds: Real,
}

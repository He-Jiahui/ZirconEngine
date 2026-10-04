use serde::{Deserialize, Serialize};

use super::{AnimationGraphClipInstance, AnimationParameterMap};

/// 单次图求值的中间结果，描述最终片段实例与遮罩目标；
/// 姿态混合和世界提交由运行时流水线继续完成。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AnimationGraphEvaluation {
    pub parameters: AnimationParameterMap,
    pub output_node: Option<String>,
    pub clips: Vec<AnimationGraphClipInstance>,
    #[serde(default)]
    pub mask_target_ids: Vec<String>,
}

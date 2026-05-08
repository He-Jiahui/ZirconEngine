use serde::{Deserialize, Serialize};

use super::{AnimationGraphClipInstance, AnimationParameterMap};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AnimationGraphEvaluation {
    pub parameters: AnimationParameterMap,
    pub output_node: Option<String>,
    pub clips: Vec<AnimationGraphClipInstance>,
    #[serde(default)]
    pub mask_target_ids: Vec<String>,
}

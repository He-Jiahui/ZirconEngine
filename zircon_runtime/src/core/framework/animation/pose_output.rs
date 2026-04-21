use serde::{Deserialize, Serialize};

use super::{AnimationPoseBone, AnimationPoseSource};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationPoseOutput {
    pub source: AnimationPoseSource,
    pub active_state: Option<String>,
    pub bones: Vec<AnimationPoseBone>,
}

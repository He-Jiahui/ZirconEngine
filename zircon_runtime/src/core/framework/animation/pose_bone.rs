use serde::{Deserialize, Serialize};

use crate::core::math::Transform;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationPoseBone {
    pub name: String,
    pub local_transform: Transform,
}

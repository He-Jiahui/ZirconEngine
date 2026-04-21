use serde::{Deserialize, Serialize};

use crate::core::math::Real;
use crate::core::resource::AssetReference;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationGraphClipInstance {
    pub clip: AssetReference,
    pub playback_speed: Real,
    pub looping: bool,
    pub weight: Real,
}

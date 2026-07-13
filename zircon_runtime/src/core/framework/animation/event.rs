use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;
use crate::core::math::Real;
use crate::core::resource::AssetReference;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationEventRecord {
    pub entity: EntityId,
    pub clip: Option<AssetReference>,
    pub target_id: Option<String>,
    pub name: String,
    pub payload: Option<String>,
    pub clip_time_seconds: Real,
    pub playback_time_seconds: Real,
}

impl AnimationEventRecord {
    pub fn new(entity: EntityId, name: impl Into<String>) -> Self {
        Self {
            entity,
            clip: None,
            target_id: None,
            name: name.into(),
            payload: None,
            clip_time_seconds: 0.0,
            playback_time_seconds: 0.0,
        }
    }

    pub fn with_clip(mut self, clip: AssetReference) -> Self {
        self.clip = Some(clip);
        self
    }

    pub fn with_target_id(mut self, target_id: impl Into<String>) -> Self {
        self.target_id = Some(target_id.into());
        self
    }

    pub fn with_payload(mut self, payload: impl Into<String>) -> Self {
        self.payload = Some(payload.into());
        self
    }

    pub fn at_times(mut self, clip_time_seconds: Real, playback_time_seconds: Real) -> Self {
        self.clip_time_seconds = clip_time_seconds;
        self.playback_time_seconds = playback_time_seconds;
        self
    }
}

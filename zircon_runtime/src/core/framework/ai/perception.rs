use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;
use crate::core::math::{Real, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiPerceptionSense {
    Sight,
    Hearing,
    Damage,
    Touch,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiHearingStimulusOrigin {
    SoundPlayback,
    AnimationEvent,
    Custom,
}

/// Neutral bus event that sound, animation, or gameplay plugins can emit without depending on a
/// concrete AI runtime implementation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiHearingStimulusEvent {
    pub source: EntityId,
    pub position: Vec3,
    pub strength: Real,
    pub max_range: Option<Real>,
    pub origin: AiHearingStimulusOrigin,
    pub age_seconds: Real,
}

impl AiHearingStimulusEvent {
    pub fn sound_playback(source: EntityId, position: Vec3, strength: Real) -> Self {
        Self {
            source,
            position,
            strength,
            max_range: None,
            origin: AiHearingStimulusOrigin::SoundPlayback,
            age_seconds: 0.0,
        }
    }

    pub fn animation_event(source: EntityId, position: Vec3, strength: Real) -> Self {
        Self {
            source,
            position,
            strength,
            max_range: None,
            origin: AiHearingStimulusOrigin::AnimationEvent,
            age_seconds: 0.0,
        }
    }

    pub fn with_max_range(mut self, max_range: Real) -> Self {
        self.max_range = Some(max_range);
        self
    }

    pub fn with_age_seconds(mut self, age_seconds: Real) -> Self {
        self.age_seconds = age_seconds;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiPerceptionStimulus {
    pub source: EntityId,
    pub sense: AiPerceptionSense,
    pub position: Vec3,
    pub strength: Real,
    pub age_seconds: Real,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AiPerceptionSnapshot {
    pub agent: EntityId,
    pub stimuli: Vec<AiPerceptionStimulus>,
}

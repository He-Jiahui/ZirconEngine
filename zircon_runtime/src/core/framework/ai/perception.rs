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

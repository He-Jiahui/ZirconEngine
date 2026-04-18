use serde::{Deserialize, Serialize};

use super::{AxisConstraint, StretchMode};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct AxisConstraintOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stretch_mode: Option<StretchMode>,
}

impl AxisConstraintOverride {
    pub fn apply_to(self, base: AxisConstraint) -> AxisConstraint {
        AxisConstraint {
            min: self.min.unwrap_or(base.min),
            max: self.max.unwrap_or(base.max),
            preferred: self.preferred.unwrap_or(base.preferred),
            priority: self.priority.unwrap_or(base.priority),
            weight: self.weight.unwrap_or(base.weight),
            stretch_mode: self.stretch_mode.unwrap_or(base.stretch_mode),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.min.is_none()
            && self.max.is_none()
            && self.preferred.is_none()
            && self.priority.is_none()
            && self.weight.is_none()
            && self.stretch_mode.is_none()
    }
}

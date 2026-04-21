use serde::{Deserialize, Serialize};

use crate::core::math::Real;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PhysicsWorldStepPlan {
    pub steps: u32,
    pub step_seconds: Real,
    pub remaining_seconds: Real,
}

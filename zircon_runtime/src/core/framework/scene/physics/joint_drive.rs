use serde::{Deserialize, Serialize};

use crate::core::math::Real;

/// Authored drive parameters for one translational or rotational joint axis.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsJointDrive {
    pub target_position: Real,
    pub target_velocity: Real,
    pub stiffness: Real,
    pub damping: Real,
    pub max_force: Real,
}

impl Default for PhysicsJointDrive {
    fn default() -> Self {
        Self {
            target_position: 0.0,
            target_velocity: 0.0,
            stiffness: 0.0,
            damping: 0.0,
            max_force: 0.0,
        }
    }
}

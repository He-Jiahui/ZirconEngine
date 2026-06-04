use serde::{Deserialize, Serialize};

use crate::core::math::Real;

use super::PhysicsJointDrive;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsJointConstraintMetadata {
    #[serde(default)]
    pub linear_limits: [Option<[Real; 2]>; 3],
    #[serde(default)]
    pub angular_limits: [Option<[Real; 2]>; 3],
    #[serde(default)]
    pub linear_drives: [PhysicsJointDrive; 3],
    #[serde(default)]
    pub angular_drives: [PhysicsJointDrive; 3],
    #[serde(default)]
    pub break_force: Option<Real>,
    #[serde(default)]
    pub break_torque: Option<Real>,
    #[serde(default)]
    pub projection_linear_tolerance: Option<Real>,
    #[serde(default)]
    pub projection_angular_tolerance: Option<Real>,
}

impl Default for PhysicsJointConstraintMetadata {
    fn default() -> Self {
        Self {
            linear_limits: [None, None, None],
            angular_limits: [None, None, None],
            linear_drives: [PhysicsJointDrive::default(); 3],
            angular_drives: [PhysicsJointDrive::default(); 3],
            break_force: None,
            break_torque: None,
            projection_linear_tolerance: None,
            projection_angular_tolerance: None,
        }
    }
}

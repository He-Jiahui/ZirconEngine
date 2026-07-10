use serde::{Deserialize, Serialize};

use crate::core::math::Real;

use super::joint_constraint_serde::{
    axis_limits_are_empty, deserialize_axis_limits, joint_drives_are_default, serialize_axis_limits,
};
use super::PhysicsJointDrive;

/// Authored joint limits and drives persisted independently from any simulation backend.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsJointConstraintMetadata {
    #[serde(
        default,
        skip_serializing_if = "axis_limits_are_empty",
        serialize_with = "serialize_axis_limits",
        deserialize_with = "deserialize_axis_limits"
    )]
    pub linear_limits: [Option<[Real; 2]>; 3],
    #[serde(
        default,
        skip_serializing_if = "axis_limits_are_empty",
        serialize_with = "serialize_axis_limits",
        deserialize_with = "deserialize_axis_limits"
    )]
    pub angular_limits: [Option<[Real; 2]>; 3],
    #[serde(default, skip_serializing_if = "joint_drives_are_default")]
    pub linear_drives: [PhysicsJointDrive; 3],
    #[serde(default, skip_serializing_if = "joint_drives_are_default")]
    pub angular_drives: [PhysicsJointDrive; 3],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub break_force: Option<Real>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub break_torque: Option<Real>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection_linear_tolerance: Option<Real>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

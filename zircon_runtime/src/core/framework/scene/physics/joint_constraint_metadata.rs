use serde::{Deserialize, Serialize};

use crate::core::math::Real;

use super::joint_constraint_serde::{
    axis_limits_are_empty, deserialize_axis_limits, joint_drives_are_default, serialize_axis_limits,
};
use super::PhysicsJointDrive;

/// Authored joint limits and drives persisted independently from any simulation backend.
///
/// 拥有轴限位、驱动及可选阈值的持久化声明，不持有后端关节对象。
/// 限位以命名轴稀疏映射写出，回读兼容旧数组；数值有限性及范围由物理插件另行校验。
/// 默认未提供限位/阈值且驱动全零，具体关节类型再决定这些槽位的用途。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsJointConstraintMetadata {
    /// 按 x/y/z 排列的线性限位，每项为可选的 `[下限, 上限]`，全缺失时省略整个字段。
    #[serde(
        default,
        skip_serializing_if = "axis_limits_are_empty",
        serialize_with = "serialize_axis_limits",
        deserialize_with = "deserialize_axis_limits"
    )]
    pub linear_limits: [Option<[Real; 2]>; 3],
    /// 按 x/y/z 排列的角限位，写出/回读规则与线性限位相同。
    #[serde(
        default,
        skip_serializing_if = "axis_limits_are_empty",
        serialize_with = "serialize_axis_limits",
        deserialize_with = "deserialize_axis_limits"
    )]
    pub angular_limits: [Option<[Real; 2]>; 3],
    /// 三个线性轴的驱动声明，全为默认驱动时省略整个字段。
    #[serde(default, skip_serializing_if = "joint_drives_are_default")]
    pub linear_drives: [PhysicsJointDrive; 3],
    /// 三个角轴的驱动声明，槽位按关节类型消费。
    #[serde(default, skip_serializing_if = "joint_drives_are_default")]
    pub angular_drives: [PhysicsJointDrive; 3],
    /// 可选断裂力阈值，缺失时保留未指定状态。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub break_force: Option<Real>,
    /// 可选断裂力矩阈值，缺失时保留未指定状态。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub break_torque: Option<Real>,
    /// 可选线性投影容差，插件校验有限且非负后再消费。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection_linear_tolerance: Option<Real>,
    /// 可选角投影容差，插件校验有限且非负后再消费。
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

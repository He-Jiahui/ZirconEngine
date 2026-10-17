use serde::{Deserialize, Serialize};

use crate::core::math::Real;

/// Authored drive parameters for one translational or rotational joint axis.
///
/// 单轴驱动的拥有型数值声明，默认五个成员全零。
/// DTO 构造和 serde 不执行准入；插件要求全部成员有限，且刚度、阻尼、最大力非负。
/// 全零驱动在关节参数转换中视为未启用，具体槽位用途见物理插件 `JointParams::from_joint_sync`。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsJointDrive {
    /// 驱动目标位置，在线性或角轴中的含义由关节类型决定。
    pub target_position: Real,
    /// 驱动目标速度，与当前轴的驱动方式配对。
    pub target_velocity: Real,
    /// 驱动刚度，插件准入要求有限且非负。
    pub stiffness: Real,
    /// 驱动阻尼，插件准入要求有限且非负。
    pub damping: Real,
    /// 驱动力上限声明，插件准入要求有限且非负。
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

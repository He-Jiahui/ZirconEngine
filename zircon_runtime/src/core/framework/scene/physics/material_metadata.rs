use serde::{Deserialize, Serialize};

use super::PhysicsCombineRule;

/// Persisted physical material properties shared by assets and scene components.
///
/// 材质标量及合并规则的拥有型持久化数据，serde 回读不会进行物理准入。
/// `Default` 将三个标量置零、两种合并规则置为平均值；成员缺失的 serde 行为不由 `Default` 自动补齐。
/// 共享插件校验只要求标量有限，提供方还可限制范围，参见物理插件的材质及 Jolt 形状校验。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PhysicsMaterialMetadata {
    /// 静摩擦系数声明，具体使用和范围由提供方决定。
    pub static_friction: f32,
    /// 动摩擦系数声明，Jolt 创建刚体时用于原生摩擦参数。
    pub dynamic_friction: f32,
    /// 恢复系数声明，Jolt 创建链要求其位于闭区间零到一。
    pub restitution: f32,
    /// 两份材质摩擦标量的合并规则声明。
    pub friction_combine: PhysicsCombineRule,
    /// 两份材质恢复系数的合并规则声明。
    pub restitution_combine: PhysicsCombineRule,
}

use serde::{Deserialize, Serialize};

/// Authored policy for combining two colliding materials' scalar properties.
///
/// 保存两份材质标量的合并策略，serde 使用蛇形小写变体名；具体应用由物理提供方承担。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhysicsCombineRule {
    /// 平均值合并策略，默认采用此项。
    #[default]
    Average,
    /// 取较小值的合并策略。
    Minimum,
    /// 取较大值的合并策略。
    Maximum,
    /// 乘积合并策略。
    Multiply,
}

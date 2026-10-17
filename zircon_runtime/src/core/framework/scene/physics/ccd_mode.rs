use serde::{Deserialize, Serialize};

/// 持久化的连续碰撞检测策略，serde 使用蛇形小写变体名。
/// 具体检测能力由物理提供方实现，Jolt 适配器将其映射为运动质量选项。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhysicsCcdMode {
    /// 使用离散检测策略，也是默认值。
    #[default]
    Disabled,
    /// 请求线性扫掠检测策略。
    LinearCast,
}

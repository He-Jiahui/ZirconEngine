use serde::{Deserialize, Serialize};

/// 持久化的刚体睡眠许可，serde 使用蛇形小写变体名。
/// 它控制是否允许睡眠，实际何时进入睡眠由物理提供方决定。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhysicsSleepPolicy {
    /// 允许提供方让刚体睡眠，默认采用此项。
    #[default]
    Allow,
    /// 请求持续禁止刚体睡眠。
    Never,
}

impl PhysicsSleepPolicy {
    /// 将可复制的许可转换为布尔值，供 Jolt 创建及运行时更新的允许睡眠选项使用。
    pub const fn allows_sleep(self) -> bool {
        matches!(self, Self::Allow)
    }
}

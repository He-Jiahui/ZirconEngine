use serde::{Deserialize, Serialize};

/// 注册表对外发布的导入与重载阶段；失败后的恢复须经资源管理器的 Reloading 边界。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceState {
    Pending,
    Ready,
    Error,
    Reloading,
}

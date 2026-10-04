use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Play 生命周期的消息投影；控制器仍持有完整状态，插件桥接按进入/离开 Playing 推导通知。
pub enum PlayStateKind {
    Edit,
    Building,
    Playing,
    CleanupFailed,
}

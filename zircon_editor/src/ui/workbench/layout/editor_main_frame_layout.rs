use serde::{Deserialize, Serialize};

use super::ActivityWindowId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 主框架仅有窗口标签与活动窗口的序列化形状；抽屉状态归各活动窗口。
pub struct EditorMainFrameLayout {
    pub active_window: ActivityWindowId,
    pub window_tabs: Vec<ActivityWindowId>,
}

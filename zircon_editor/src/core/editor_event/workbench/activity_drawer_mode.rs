use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 活动抽屉的布局状态；布局管理器将其用于展开、自动隐藏和折叠策略，持久化布局沿用这些值。
pub enum ActivityDrawerMode {
    Pinned,
    AutoHide,
    Collapsed,
}

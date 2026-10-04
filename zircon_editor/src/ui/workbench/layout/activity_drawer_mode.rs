use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 抽屉显示策略；折叠保留成员，但清除当前活动视图。
pub enum ActivityDrawerMode {
    Pinned,
    AutoHide,
    Collapsed,
}

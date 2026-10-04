use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// 活动窗口内的逻辑停靠槽；同侧上下两槽共享展示区域。
pub enum ActivityDrawerSlot {
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom,
    Bottom,
}

impl ActivityDrawerSlot {
    pub const ALL: [Self; 5] = [
        Self::LeftTop,
        Self::LeftBottom,
        Self::RightTop,
        Self::RightBottom,
        Self::Bottom,
    ];

    pub fn is_bottom(self) -> bool {
        self == Self::Bottom
    }

    /// 决定激活抽屉时应折叠哪些兄弟；不比较跨窗口归属。
    pub fn shares_region(self, other: Self) -> bool {
        matches!(
            (self, other),
            (
                Self::LeftTop | Self::LeftBottom,
                Self::LeftTop | Self::LeftBottom
            ) | (
                Self::RightTop | Self::RightBottom,
                Self::RightTop | Self::RightBottom
            ) | (Self::Bottom, Self::Bottom)
        )
    }
}

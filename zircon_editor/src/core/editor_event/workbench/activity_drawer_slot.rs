use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// 工作台固定抽屉位置标识；布局、窗口注册及投影共同使用，ALL 提供稳定遍历顺序。
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
}

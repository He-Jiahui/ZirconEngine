use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ActivityDrawerSlot {
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom,
    BottomLeft,
    BottomRight,
}

impl ActivityDrawerSlot {
    pub const ALL: [Self; 6] = [
        Self::LeftTop,
        Self::LeftBottom,
        Self::RightTop,
        Self::RightBottom,
        Self::BottomLeft,
        Self::BottomRight,
    ];
}

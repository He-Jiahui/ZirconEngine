use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

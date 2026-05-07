use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ActivityDrawerSlot {
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom,
    Bottom,
    BottomLeft,
    BottomRight,
}

impl ActivityDrawerSlot {
    pub const ALL: [Self; 5] = [
        Self::LeftTop,
        Self::LeftBottom,
        Self::RightTop,
        Self::RightBottom,
        Self::Bottom,
    ];

    pub fn canonical(self) -> Self {
        match self {
            Self::BottomLeft | Self::BottomRight => Self::Bottom,
            slot => slot,
        }
    }

    pub fn is_bottom(self) -> bool {
        matches!(self, Self::Bottom | Self::BottomLeft | Self::BottomRight)
    }
}

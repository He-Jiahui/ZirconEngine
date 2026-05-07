use serde::{Deserialize, Serialize};

use crate::ui::workbench::layout::ActivityDrawerSlot;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DrawerDockPosition {
    LeftTop,
    LeftBottom,
    Bottom,
    RightTop,
    RightBottom,
}

impl DrawerDockPosition {
    pub fn primary_slot(self) -> ActivityDrawerSlot {
        match self {
            Self::LeftTop => ActivityDrawerSlot::LeftTop,
            Self::LeftBottom => ActivityDrawerSlot::LeftBottom,
            Self::Bottom => ActivityDrawerSlot::Bottom,
            Self::RightTop => ActivityDrawerSlot::RightTop,
            Self::RightBottom => ActivityDrawerSlot::RightBottom,
        }
    }

    pub fn from_slot(slot: ActivityDrawerSlot) -> Self {
        match slot {
            ActivityDrawerSlot::LeftTop => Self::LeftTop,
            ActivityDrawerSlot::LeftBottom => Self::LeftBottom,
            ActivityDrawerSlot::RightTop => Self::RightTop,
            ActivityDrawerSlot::RightBottom => Self::RightBottom,
            ActivityDrawerSlot::Bottom
            | ActivityDrawerSlot::BottomLeft
            | ActivityDrawerSlot::BottomRight => Self::Bottom,
        }
    }
}

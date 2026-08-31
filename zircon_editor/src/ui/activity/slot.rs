use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityDrawerSlotPreference {
    LeftTop,
    LeftBottom,
    Bottom,
    RightTop,
    RightBottom,
}

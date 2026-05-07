use crate::ui::ActivityDrawerSlotPreference;

use crate::ui::workbench::layout::ActivityDrawerSlot;

pub(super) fn drawer_slot_preference(slot: ActivityDrawerSlot) -> ActivityDrawerSlotPreference {
    match slot {
        ActivityDrawerSlot::LeftTop => ActivityDrawerSlotPreference::LeftTop,
        ActivityDrawerSlot::LeftBottom => ActivityDrawerSlotPreference::LeftBottom,
        ActivityDrawerSlot::RightTop => ActivityDrawerSlotPreference::RightTop,
        ActivityDrawerSlot::RightBottom => ActivityDrawerSlotPreference::RightBottom,
        ActivityDrawerSlot::Bottom
        | ActivityDrawerSlot::BottomLeft
        | ActivityDrawerSlot::BottomRight => ActivityDrawerSlotPreference::Bottom,
    }
}

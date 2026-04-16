use zircon_editor_ui::ActivityDrawerSlotPreference;

use crate::layout::ActivityDrawerSlot;

pub(super) fn drawer_slot_preference(slot: ActivityDrawerSlot) -> ActivityDrawerSlotPreference {
    match slot {
        ActivityDrawerSlot::LeftTop => ActivityDrawerSlotPreference::LeftTop,
        ActivityDrawerSlot::LeftBottom => ActivityDrawerSlotPreference::LeftBottom,
        ActivityDrawerSlot::RightTop => ActivityDrawerSlotPreference::RightTop,
        ActivityDrawerSlot::RightBottom => ActivityDrawerSlotPreference::RightBottom,
        ActivityDrawerSlot::BottomLeft => ActivityDrawerSlotPreference::BottomLeft,
        ActivityDrawerSlot::BottomRight => ActivityDrawerSlotPreference::BottomRight,
    }
}

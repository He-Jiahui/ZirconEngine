use crate::ui::workbench::layout::ActivityDrawerSlot;

pub(super) fn drawer_slot_key(slot: ActivityDrawerSlot) -> &'static str {
    match slot {
        ActivityDrawerSlot::LeftTop => "left_top",
        ActivityDrawerSlot::LeftBottom => "left_bottom",
        ActivityDrawerSlot::RightTop => "right_top",
        ActivityDrawerSlot::RightBottom => "right_bottom",
        ActivityDrawerSlot::Bottom
        | ActivityDrawerSlot::BottomLeft
        | ActivityDrawerSlot::BottomRight => "bottom",
    }
}

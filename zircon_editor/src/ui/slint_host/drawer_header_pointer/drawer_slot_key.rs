pub(super) fn drawer_slot_key(slot: crate::ActivityDrawerSlot) -> &'static str {
    match slot {
        crate::ActivityDrawerSlot::LeftTop => "left_top",
        crate::ActivityDrawerSlot::LeftBottom => "left_bottom",
        crate::ActivityDrawerSlot::RightTop => "right_top",
        crate::ActivityDrawerSlot::RightBottom => "right_bottom",
        crate::ActivityDrawerSlot::BottomLeft => "bottom_left",
        crate::ActivityDrawerSlot::BottomRight => "bottom_right",
    }
}

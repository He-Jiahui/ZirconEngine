pub(super) fn drawer_slot_key(
    slot: crate::ui::workbench::layout::ActivityDrawerSlot,
) -> &'static str {
    match slot {
        crate::ui::workbench::layout::ActivityDrawerSlot::LeftTop => "left_top",
        crate::ui::workbench::layout::ActivityDrawerSlot::LeftBottom => "left_bottom",
        crate::ui::workbench::layout::ActivityDrawerSlot::RightTop => "right_top",
        crate::ui::workbench::layout::ActivityDrawerSlot::RightBottom => "right_bottom",
        crate::ui::workbench::layout::ActivityDrawerSlot::BottomLeft => "bottom_left",
        crate::ui::workbench::layout::ActivityDrawerSlot::BottomRight => "bottom_right",
    }
}

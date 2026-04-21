use crate::ui::workbench::layout::ActivityDrawerSlot;

pub(crate) fn parse_activity_drawer_slot(slot: &str) -> Result<ActivityDrawerSlot, String> {
    match slot {
        "left_top" => Ok(ActivityDrawerSlot::LeftTop),
        "left_bottom" => Ok(ActivityDrawerSlot::LeftBottom),
        "right_top" => Ok(ActivityDrawerSlot::RightTop),
        "right_bottom" => Ok(ActivityDrawerSlot::RightBottom),
        "bottom_left" => Ok(ActivityDrawerSlot::BottomLeft),
        "bottom_right" => Ok(ActivityDrawerSlot::BottomRight),
        _ => Err(format!("unknown drawer slot {slot}")),
    }
}

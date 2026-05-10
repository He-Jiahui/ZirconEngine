use crate::ui::workbench::layout::ActivityDrawerSlot;

pub(crate) fn parse_activity_drawer_slot(slot: &str) -> Result<ActivityDrawerSlot, String> {
    match slot {
        "left_top" => Ok(ActivityDrawerSlot::LeftTop),
        "left_bottom" => Ok(ActivityDrawerSlot::LeftBottom),
        "right_top" => Ok(ActivityDrawerSlot::RightTop),
        "right_bottom" => Ok(ActivityDrawerSlot::RightBottom),
        "bottom" | "bottom_left" | "bottom_right" => Ok(ActivityDrawerSlot::Bottom),
        _ => Err(format!("unknown drawer slot {slot}")),
    }
}

use super::super::error::EditorBindingDispatchError;
use crate::core::editor_event::ActivityDrawerSlot;

pub(super) fn parse_drawer_slot(
    slot: &str,
) -> Result<ActivityDrawerSlot, EditorBindingDispatchError> {
    match slot {
        "left_top" => Ok(ActivityDrawerSlot::LeftTop),
        "left_bottom" => Ok(ActivityDrawerSlot::LeftBottom),
        "right_top" => Ok(ActivityDrawerSlot::RightTop),
        "right_bottom" => Ok(ActivityDrawerSlot::RightBottom),
        "bottom" | "bottom_left" | "bottom_right" => Ok(ActivityDrawerSlot::Bottom),
        _ => Err(EditorBindingDispatchError::UnknownDrawerSlot(
            slot.to_string(),
        )),
    }
}

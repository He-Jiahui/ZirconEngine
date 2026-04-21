use super::super::error::EditorBindingDispatchError;
use crate::core::editor_event::ActivityDrawerMode;

pub(super) fn parse_drawer_mode(
    mode: &str,
) -> Result<ActivityDrawerMode, EditorBindingDispatchError> {
    match mode {
        "Pinned" => Ok(ActivityDrawerMode::Pinned),
        "AutoHide" => Ok(ActivityDrawerMode::AutoHide),
        "Collapsed" => Ok(ActivityDrawerMode::Collapsed),
        _ => Err(EditorBindingDispatchError::UnknownDrawerMode(
            mode.to_string(),
        )),
    }
}

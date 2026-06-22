pub(super) fn viewport_toolbar_click_affects_viewport_or_status(control_id: &str) -> bool {
    matches!(
        control_id,
        "EnterPlayMode" | "ExitPlayMode" | "frame.selection" | "frame_selection"
    ) || control_id.starts_with("align.")
}

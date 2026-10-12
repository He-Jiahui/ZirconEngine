pub(in crate::ui::retained_host::app) fn close_action_id(action: &str) -> Option<&'static str> {
    match action {
        "save" => Some("save"),
        "discard" => Some("discard"),
        "cancel" => Some("cancel"),
        _ => None,
    }
}

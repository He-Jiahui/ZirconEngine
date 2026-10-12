pub(super) fn take_required_id(id: Option<String>) -> String {
    id.expect("optional feature should declare id")
}

pub(super) fn take_required_display_name(display_name: Option<String>) -> String {
    display_name.expect("optional feature should declare display name")
}

pub(super) fn take_required_owner_plugin_id(owner_plugin_id: Option<String>) -> String {
    owner_plugin_id.expect("optional feature should declare owner plugin id")
}

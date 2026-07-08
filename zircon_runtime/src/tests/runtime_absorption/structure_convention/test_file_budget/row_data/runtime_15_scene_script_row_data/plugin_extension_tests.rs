use super::*;

#[path = "plugin_extension_tests/child_rows.rs"]
mod child_rows;
#[path = "plugin_extension_tests/export_chain.rs"]
mod export_chain;
#[path = "plugin_extension_tests/folder_backed.rs"]
mod folder_backed;
#[path = "plugin_extension_tests/status_mirrors.rs"]
mod status_mirrors;

#[test]
fn runtime_15_scene_script_plugin_extension_row_data_is_child_backed() {
    child_rows::assert_plugin_extension_child_rows_are_route_owned();
    export_chain::assert_plugin_extension_exports_are_current();
    status_mirrors::assert_plugin_extension_row_data_status_is_current();
}

#[test]
fn runtime_15_scene_script_plugin_extension_guard_is_folder_backed() {
    folder_backed::assert_plugin_extension_guard_is_folder_backed();
    status_mirrors::assert_plugin_extension_guard_status_is_current();
}

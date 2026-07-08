use super::*;

pub(super) fn assert_plugin_extension_guard_is_folder_backed() {
    let guard_parent = read_runtime_src(SCENE_SCRIPT_PLUGIN_EXTENSION_GUARD_PATH);

    assert_contains_all(
        "plugin-extension guard route mounts folder-backed children",
        &guard_parent,
        &[
            "#[path = \"plugin_extension_tests/child_rows.rs\"]",
            "#[path = \"plugin_extension_tests/export_chain.rs\"]",
            "#[path = \"plugin_extension_tests/folder_backed.rs\"]",
            "#[path = \"plugin_extension_tests/status_mirrors.rs\"]",
            "child_rows::assert_plugin_extension_child_rows_are_route_owned();",
            "export_chain::assert_plugin_extension_exports_are_current();",
            "status_mirrors::assert_plugin_extension_row_data_status_is_current();",
            "folder_backed::assert_plugin_extension_guard_is_folder_backed();",
            "status_mirrors::assert_plugin_extension_guard_status_is_current();",
        ],
    );
    for moved_marker in [
        "let plugin_route = read_runtime_src",
        "let scene_parent = read_runtime_src",
        "M3 status map records plugin-extension row-data child split",
        "Runtime 15 M3 runtime plugin lifecycle fixture child-owner split",
    ] {
        assert!(
            !guard_parent.contains(moved_marker),
            "plugin_extension_tests.rs should delegate {moved_marker} to child guard files"
        );
    }
    for (_, path, marker) in PLUGIN_EXTENSION_GUARD_CHILDREN {
        let child_source = read_runtime_src(path);
        assert_contains_all(
            "plugin-extension guard child keeps representative assertion",
            &child_source,
            &[*marker],
        );
        assert!(
            child_source.lines().count() < 100,
            "{path} should stay below its focused plugin-extension guard budget"
        );
    }
}

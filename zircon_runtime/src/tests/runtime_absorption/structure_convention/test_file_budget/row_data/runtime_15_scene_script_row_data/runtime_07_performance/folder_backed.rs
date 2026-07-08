use super::*;

pub(super) fn assert_runtime_07_performance_guard_is_folder_backed() {
    let guard_parent = read_runtime_src(SCENE_SCRIPT_RUNTIME_07_PERFORMANCE_GUARD_PATH);

    assert_contains_all(
        "Runtime 07 performance guard route mounts folder-backed children",
        &guard_parent,
        &[
            "#[path = \"runtime_07_performance/child_rows.rs\"]",
            "#[path = \"runtime_07_performance/export_chain.rs\"]",
            "#[path = \"runtime_07_performance/folder_backed.rs\"]",
            "#[path = \"runtime_07_performance/status_mirrors.rs\"]",
            "child_rows::assert_runtime_07_performance_child_rows_are_route_owned();",
            "export_chain::assert_runtime_07_performance_exports_are_current();",
            "status_mirrors::assert_runtime_07_performance_row_data_status_is_current();",
            "folder_backed::assert_runtime_07_performance_guard_is_folder_backed();",
            "status_mirrors::assert_runtime_07_performance_guard_status_is_current();",
        ],
    );
    for moved_marker in [
        "let runtime_07_route = read_runtime_src",
        "let scene_script_parent = read_runtime_src",
        "M3 status map records Runtime 07 performance row-data child split",
        "Runtime 15 M3 Runtime 07 owner-budget mirror-docs guard folder-backed split",
    ] {
        assert!(
            !guard_parent.contains(moved_marker),
            "runtime_07_performance.rs should delegate {moved_marker} to child guard files"
        );
    }
    for (_, path, marker) in RUNTIME_07_PERFORMANCE_GUARD_CHILDREN {
        let child_source = read_runtime_src(path);
        assert_contains_all(
            "Runtime 07 performance guard child keeps representative assertion",
            &child_source,
            &[*marker],
        );
        assert!(
            child_source.lines().count() < 100,
            "{path} should stay below its focused Runtime 07 performance guard budget"
        );
    }
}

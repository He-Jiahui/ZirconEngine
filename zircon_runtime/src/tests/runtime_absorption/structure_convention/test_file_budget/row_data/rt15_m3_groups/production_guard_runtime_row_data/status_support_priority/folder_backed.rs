use super::*;

const STATUS_SUPPORT_PRIORITY_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_rows",
        PRODUCTION_GUARD_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_CHILD_ROWS_GUARD_PATH,
        "status-support priority route mounts child row groups",
    ),
    (
        "export_chain",
        PRODUCTION_GUARD_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_EXPORT_CHAIN_GUARD_PATH,
        "production guard runtime row-data exports status-support priority child groups",
    ),
    (
        "folder_backed",
        PRODUCTION_GUARD_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_FOLDER_BACKED_GUARD_PATH,
        "status-support priority guard route mounts folder-backed children",
    ),
    (
        "status_mirrors",
        PRODUCTION_GUARD_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_STATUS_MIRRORS_GUARD_PATH,
        "PRODUCTION_GUARD_STATUS_SUPPORT_PRIORITY_GUARD_FOLDER_BACKED_STATUS_ID",
    ),
];

pub(super) fn assert_status_support_priority_guard_is_folder_backed() {
    let guard_parent =
        read_runtime_src(PRODUCTION_GUARD_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_GUARD_PATH);

    assert_contains_all(
        "status-support priority guard route mounts folder-backed children",
        &guard_parent,
        &[
            "#[path = \"status_support_priority/child_rows.rs\"]",
            "#[path = \"status_support_priority/export_chain.rs\"]",
            "#[path = \"status_support_priority/folder_backed.rs\"]",
            "#[path = \"status_support_priority/status_mirrors.rs\"]",
            "child_rows::assert_status_support_priority_child_rows_are_route_owned();",
            "export_chain::assert_status_support_priority_exports_are_current();",
            "status_mirrors::assert_status_support_priority_row_data_status_is_current();",
            "folder_backed::assert_status_support_priority_guard_is_folder_backed();",
            "status_mirrors::assert_status_support_priority_guard_status_is_current();",
        ],
    );
    for moved_marker in [
        "let route = read_runtime_src",
        "let runtime_row_data = read_runtime_src",
        "M3 status map records status-support priority child split",
        "Runtime 15 M3 priority plan docs row-data guard folder-backed split",
    ] {
        assert!(
            !guard_parent.contains(moved_marker),
            "status_support_priority.rs should delegate {moved_marker} to child guard files"
        );
    }
    for (_, path, marker) in STATUS_SUPPORT_PRIORITY_GUARD_CHILDREN {
        let child_source = read_runtime_src(path);
        assert_contains_all(
            "status-support priority guard child keeps representative assertion",
            &child_source,
            &[*marker],
        );
        assert!(
            child_source.lines().count() < 100,
            "{path} should stay below its focused status-support priority guard budget"
        );
    }
}

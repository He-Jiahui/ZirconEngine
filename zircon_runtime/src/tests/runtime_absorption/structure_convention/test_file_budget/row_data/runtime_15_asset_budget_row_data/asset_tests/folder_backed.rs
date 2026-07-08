use super::*;

pub(super) fn assert_asset_tests_guard_is_folder_backed() {
    let guard_parent = read_runtime_src(ASSET_BUDGET_ASSET_TESTS_GUARD_PATH);

    assert_contains_all(
        "asset-tests guard route mounts folder-backed children",
        &guard_parent,
        &[
            "#[path = \"asset_tests/child_rows.rs\"]",
            "#[path = \"asset_tests/export_chain.rs\"]",
            "#[path = \"asset_tests/folder_backed.rs\"]",
            "#[path = \"asset_tests/status_mirrors.rs\"]",
            "child_rows::assert_asset_tests_child_rows_are_route_owned();",
            "export_chain::assert_asset_tests_exports_are_current();",
            "status_mirrors::assert_asset_tests_row_data_status_is_current();",
            "folder_backed::assert_asset_tests_guard_is_folder_backed();",
            "status_mirrors::assert_asset_tests_guard_status_is_current();",
        ],
    );
    for moved_marker in [
        "let asset_tests_route = read_runtime_src",
        "let asset_budget_parent = read_runtime_src",
        "M3 status map records asset-tests row-data child split",
        "Runtime 15 M3 asset artifact store test folder split",
    ] {
        assert!(
            !guard_parent.contains(moved_marker),
            "asset_tests.rs should delegate {moved_marker} to child guard files"
        );
    }
    for (_, path, marker) in ASSET_TESTS_GUARD_CHILDREN {
        let child_source = read_runtime_src(path);
        assert_contains_all(
            "asset-tests guard child keeps representative assertion",
            &child_source,
            &[*marker],
        );
        assert!(
            child_source.lines().count() < 100,
            "{path} should stay below its focused asset-tests guard budget"
        );
    }
}

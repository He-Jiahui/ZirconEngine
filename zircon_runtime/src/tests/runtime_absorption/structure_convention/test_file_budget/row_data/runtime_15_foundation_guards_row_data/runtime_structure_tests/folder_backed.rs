use super::*;

pub(super) fn assert_runtime_structure_guard_is_folder_backed() {
    let guard_parent = read_runtime_src(FOUNDATION_GUARDS_RUNTIME_STRUCTURE_GUARD_PATH);

    assert_contains_all(
        "runtime-structure guard route mounts folder-backed children",
        &guard_parent,
        &[
            "#[path = \"runtime_structure_tests/child_rows.rs\"]",
            "#[path = \"runtime_structure_tests/export_chain.rs\"]",
            "#[path = \"runtime_structure_tests/folder_backed.rs\"]",
            "#[path = \"runtime_structure_tests/status_mirrors.rs\"]",
            "child_rows::assert_runtime_structure_child_rows_are_route_owned();",
            "export_chain::assert_runtime_structure_exports_are_current();",
            "status_mirrors::assert_runtime_structure_row_data_status_is_current();",
            "folder_backed::assert_runtime_structure_guard_is_folder_backed();",
            "status_mirrors::assert_runtime_structure_guard_status_is_current();",
        ],
    );
    for moved_marker in [
        "let route_parent = read_runtime_src",
        "let foundation_guards_parent = read_runtime_src",
        "M3 status map records runtime-structure row-data child split",
        "Runtime 15 M3 root entries guard child-owner split",
    ] {
        assert!(
            !guard_parent.contains(moved_marker),
            "runtime_structure_tests.rs should delegate {moved_marker} to child guard files"
        );
    }
    for (_, path, marker) in RUNTIME_STRUCTURE_GUARD_CHILDREN {
        let child_source = read_runtime_src(path);
        assert_contains_all(
            "runtime-structure guard child keeps representative assertion",
            &child_source,
            &[*marker],
        );
        assert!(
            child_source.lines().count() < 100,
            "{path} should stay below its focused runtime-structure guard budget"
        );
    }
}

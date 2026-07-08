use super::*;

const RUNTIME_STRUCTURE_CHILD_ROWS: &[(&str, &str, &str)] = &[
    (
        "core_runtime_rows",
        FOUNDATION_GUARDS_RUNTIME_STRUCTURE_CORE_RUNTIME_ROWS_PATH,
        "Runtime 15 M3 core runtime registration structure owner split",
    ),
    (
        "root_route_rows",
        FOUNDATION_GUARDS_RUNTIME_STRUCTURE_ROOT_ROUTE_ROWS_PATH,
        "Runtime 15 M3 root entries guard child-owner split",
    ),
    (
        "runtime_absorption_core_rows",
        FOUNDATION_GUARDS_RUNTIME_STRUCTURE_RUNTIME_ABSORPTION_CORE_ROWS_PATH,
        "Runtime 15 M3 job-system route-owner split",
    ),
    (
        "runtime_absorption_platform_rows",
        FOUNDATION_GUARDS_RUNTIME_STRUCTURE_RUNTIME_ABSORPTION_PLATFORM_ROWS_PATH,
        "Runtime 15 M3 dynamic-scene route-owner split",
    ),
    (
        "test_guard_rows",
        FOUNDATION_GUARDS_RUNTIME_STRUCTURE_TEST_GUARD_ROWS_PATH,
        "Runtime 15 M3 diagnostics guard module split",
    ),
    (
        "row_data_owner",
        FOUNDATION_GUARDS_RUNTIME_STRUCTURE_ROW_DATA_OWNER_PATH,
        RUNTIME_STRUCTURE_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
    ),
];

pub(super) fn assert_runtime_structure_child_rows_are_route_owned() {
    let route_parent = read_runtime_src(FOUNDATION_GUARDS_RUNTIME_STRUCTURE_TESTS_PATH);

    assert_contains_all(
        "runtime-structure row-data route mounts child row groups",
        &route_parent,
        &[
            "runtime_structure_tests/core_runtime_rows.rs",
            "runtime_structure_tests/root_route_rows.rs",
            "runtime_structure_tests/runtime_absorption_core_rows.rs",
            "runtime_structure_tests/runtime_absorption_platform_rows.rs",
            "runtime_structure_tests/test_guard_rows.rs",
            "runtime_structure_tests/row_data_owner.rs",
            "core_runtime_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "root_route_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_absorption_core_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_absorption_platform_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "test_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    for moved_row in [
        "Runtime 15 M3 root entries guard child-owner split",
        "Runtime 15 M3 job-system route-owner split",
        "Runtime 15 M3 dynamic-scene route-owner split",
        "Runtime 15 M3 diagnostics guard module split",
    ] {
        assert!(
            !route_parent.contains(moved_row),
            "runtime_structure_tests.rs should delegate {moved_row} to child row files"
        );
    }
    for (module_name, path, representative_row) in RUNTIME_STRUCTURE_CHILD_ROWS {
        let child_source = read_runtime_src(path);
        assert_contains_all(
            "runtime-structure child row file keeps representative row",
            &child_source,
            &[*representative_row],
        );
        assert!(
            route_parent.contains(&format!("mod {module_name};")),
            "runtime_structure_tests.rs should mount {module_name}"
        );
        assert!(
            child_source.lines().count() < 130,
            "{path} should stay below its focused row-data budget"
        );
    }
}

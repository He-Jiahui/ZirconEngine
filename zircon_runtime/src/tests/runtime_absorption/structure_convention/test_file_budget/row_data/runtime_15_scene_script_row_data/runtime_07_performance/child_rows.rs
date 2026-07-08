use super::*;

const RUNTIME_07_PERFORMANCE_CHILD_ROWS: &[(&str, &str, &str)] = &[
    (
        "primary_guard_rows",
        SCENE_SCRIPT_RUNTIME_07_PRIMARY_GUARD_ROWS_PATH,
        "Runtime 15 M3 Runtime 07 performance hotspot guard folder split",
    ),
    (
        "split_layout_rows",
        SCENE_SCRIPT_RUNTIME_07_SPLIT_LAYOUT_ROWS_PATH,
        "Runtime 15 M3 Runtime 07 hotspot-inventory split-layout guard folder-backed split",
    ),
    (
        "owner_budget_rows",
        SCENE_SCRIPT_RUNTIME_07_OWNER_BUDGET_ROWS_PATH,
        "Runtime 15 M3 Runtime 07 owner-budget mirror-docs guard folder-backed split",
    ),
    (
        "row_data_owner",
        SCENE_SCRIPT_RUNTIME_07_ROW_DATA_OWNER_PATH,
        RUNTIME_07_PERFORMANCE_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
    ),
];

pub(super) fn assert_runtime_07_performance_child_rows_are_route_owned() {
    let runtime_07_route = read_runtime_src(SCENE_SCRIPT_RUNTIME_07_PERFORMANCE_PATH);

    assert_contains_all(
        "Runtime 07 performance row-data route mounts child row groups",
        &runtime_07_route,
        &[
            "runtime_07_performance/primary_guard_rows.rs",
            "runtime_07_performance/split_layout_rows.rs",
            "runtime_07_performance/owner_budget_rows.rs",
            "runtime_07_performance/row_data_owner.rs",
            "primary_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "split_layout_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "owner_budget_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    for moved_row in [
        "Runtime 15 M3 Runtime 07 hotspot-inventory split-layout guard folder-backed split",
        "Runtime 15 M3 Runtime 07 owner-budget mirror-docs guard folder-backed split",
        "Runtime 15 M3 Runtime 07 owner-budget mirror-docs child-owner split",
    ] {
        assert!(
            !runtime_07_route.contains(moved_row),
            "runtime_07_performance.rs should delegate {moved_row} to child row files"
        );
    }
    for (module_name, path, representative_row) in RUNTIME_07_PERFORMANCE_CHILD_ROWS {
        let child_source = read_runtime_src(path);
        assert_contains_all(
            "Runtime 07 performance child row file keeps representative row",
            &child_source,
            &[*representative_row],
        );
        assert!(
            runtime_07_route.contains(&format!("mod {module_name};")),
            "runtime_07_performance.rs should mount {module_name}"
        );
        assert!(
            child_source.lines().count() < 130,
            "{path} should stay below its focused row-data budget"
        );
    }
}

use super::*;

pub(super) fn assert_expected_slice_owner_path_budget_traversal_is_current() {
    let route = route_children::expected_slice_owner_path_route_source();
    let root_owner_paths = read_runtime_src(ROOT_OWNER_PATHS_PATH);
    let budgets = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/budgets.rs",
    );

    for moved_anchor in [
        "support_layout_rows.rs",
        "route_metadata/naming_boundary_rows.rs",
        "structure_support/parent_route_rows.rs",
        "status_support_maps/route_guard_rows.rs",
        "review_guard_structure/root_route_rows/route_metadata_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/warning_cleanup.rs",
    ] {
        assert!(
            !route.contains(moved_anchor),
            "expected_slice_maps.rs should not retain moved owner-path tuple for {moved_anchor}"
        );
    }
    assert_contains_all(
        "status-support root owner path traversal expands expected-slice child groups",
        &root_owner_paths,
        &[
            "status_support_row_owner_path_groups",
            "STATUS_SUPPORT_EXPECTED_SLICE_MAP_OWNER_PATH_GROUPS",
            ".chain(",
        ],
    );
    assert_contains_all(
        "status-support row-data budget traverses owner path group iterator",
        &budgets,
        &["status_support_row_owner_path_groups()"],
    );
    assert!(
        !budgets.contains("STATUS_SUPPORT_ROW_OWNER_PATH_GROUPS"),
        "budgets.rs should not depend on the old flat owner path group constant"
    );
}

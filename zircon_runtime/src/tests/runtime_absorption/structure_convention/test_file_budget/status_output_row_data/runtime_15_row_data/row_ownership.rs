use super::*;

#[path = "row_ownership/foundation_rows.rs"]
mod foundation_rows;
#[path = "row_ownership/group_exports.rs"]
mod group_exports;
#[path = "row_ownership/owner_budgets.rs"]
mod owner_budgets;
#[path = "row_ownership/status_support.rs"]
mod status_support;

const ROW_OWNERSHIP_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "foundation_rows",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership/foundation_rows.rs",
        "runtime_15_row_data_foundation_rows_are_child_owned",
    ),
    (
        "group_exports",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership/group_exports.rs",
        "runtime_15_row_data_group_exports_are_child_owned",
    ),
    (
        "owner_budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership/owner_budgets.rs",
        "runtime_15_row_data_owner_budgets_are_child_owned",
    ),
    (
        "status_support",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership/status_support.rs",
        "runtime_15_row_data_status_support_rows_are_child_owned",
    ),
];

#[test]
fn runtime_15_status_output_runtime_15_row_data_is_child_owner() {
    let route = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership.rs",
    );

    for (module, path, guard) in ROW_OWNERSHIP_CHILDREN {
        assert!(
            route.contains(&format!("#[path = \"row_ownership/{module}.rs\"]")),
            "row_ownership.rs should route {module} to its child file"
        );
        assert!(
            route.contains(&format!("mod {module};")),
            "row_ownership.rs should mount {module}"
        );

        let child = read_runtime_src(path);
        assert!(
            child.contains(guard),
            "{path} should own the {guard} assertion"
        );
        assert!(
            child.lines().count() < 90,
            "{path} should stay focused after row_ownership split"
        );
    }
}

#[test]
fn runtime_15_runtime_15_row_data_row_ownership_children_are_child_owned() {
    let route = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership.rs",
    );
    let row_data = read_runtime_src(RUNTIME_15_M3_STATUS_SUPPORT_ROW_DATA_AND_BUDGET_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    assert_contains_all(
        "Runtime 15 row-ownership split is recorded in status rows",
        &row_data,
        &[
            ROW_OWNERSHIP_CHILD_SPLIT_STATUS_NAME,
            ROW_OWNERSHIP_CHILD_SPLIT_STATUS_ID,
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership/group_exports.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership/foundation_rows.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership/status_support.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership/owner_budgets.rs",
            ROW_OWNERSHIP_CHILD_SPLIT_GUARD_NAME,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 row-ownership split is mirrored in status/date maps",
        &(status_map + &date_map),
        &[
            ROW_OWNERSHIP_CHILD_SPLIT_STATUS_NAME,
            ROW_OWNERSHIP_CHILD_SPLIT_STATUS_ID,
            "2026-07-04",
        ],
    );
    assert!(
        route.lines().count() < 110,
        "row_ownership.rs should remain a routing/status guard after child split"
    );
}

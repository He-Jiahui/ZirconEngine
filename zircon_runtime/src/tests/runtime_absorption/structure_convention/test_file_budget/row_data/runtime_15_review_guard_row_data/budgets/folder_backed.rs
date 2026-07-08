use super::*;

const REVIEW_GUARD_ROW_DATA_BUDGETS_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/base_child_owner_maps.rs";
const REVIEW_GUARD_ROW_DATA_BUDGETS_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/base_child_owner_maps.rs";

#[test]
fn runtime_15_review_guard_row_data_budgets_guard_is_folder_backed() {
    let parent = read_runtime_src(REVIEW_GUARD_ROW_DATA_BUDGETS_PATH);
    let status_rows = read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_ROWS_PATH);
    let status_map = read_runtime_src(REVIEW_GUARD_ROW_DATA_BUDGETS_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_ROW_DATA_BUDGETS_DATE_MAP_PATH);

    for module_name in REVIEW_GUARD_ROW_DATA_BUDGET_CHILDREN {
        let module_mount = format!("#[path = \"budgets/{module_name}.rs\"]");
        assert_contains_all(
            "review-guard row-data budgets route mounts child budget groups",
            &parent,
            &[module_mount.as_str(), &format!("mod {module_name};")],
        );
        assert_contains_all(
            "review-guard row-data budget child owns focused budget checks",
            &read_runtime_src(&review_guard_row_data_budget_child_path(module_name)),
            &["#[test]", "assert_runtime_15_review_guard_row_data_budgets"],
        );
    }

    for moved_anchor in [
        "(DELEGATION_GUARD_PATH, 35)",
        "(STATUS_SUPPORT_ROWS_GUARD_PATH, 30)",
        "(TYPED_ERROR_ROWS_GUARD_PATH, 40)",
        "(ROOT_PATHS_PATH, 190)",
        "REVIEW_GUARD_ROW_DATA_AGGREGATION_CHILDREN",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "review-guard row-data budgets route should not own moved budget anchor {moved_anchor}"
        );
    }

    assert_runtime_15_review_guard_row_data_budgets(&[
        (REVIEW_GUARD_ROW_DATA_GUARD_PATH, 150),
        (REVIEW_GUARD_ROW_DATA_BUDGETS_PATH, 80),
        (&review_guard_row_data_budget_child_path("delegation"), 60),
        (
            &review_guard_row_data_budget_child_path("folder_backed"),
            110,
        ),
        (&review_guard_row_data_budget_child_path("root_rows"), 100),
        (
            &review_guard_row_data_budget_child_path("status_support_rows"),
            100,
        ),
        (
            &review_guard_row_data_budget_child_path("typed_error_rows"),
            80,
        ),
    ]);
    assert_contains_all(
        "review-guard status rows record budgets guard folder-backed split",
        &status_rows,
        &[
            "Runtime 15 M3 review-guard row-data budgets guard folder-backed split",
            "runtime_15_review_guard_row_data_budgets_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_review_guard_row_data_budgets_guard_is_folder_backed",
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "review-guard row-data status/date maps record budgets guard folder-backed split",
        &(status_map + &date_map),
        &[
            "Runtime 15 M3 review-guard row-data budgets guard folder-backed split",
            "runtime_15_review_guard_row_data_budgets_guard_folder_backed_static_passed_cargo_deferred",
            "2026-07-06",
        ],
    );
}

use super::*;

#[test]
fn runtime_15_review_guard_code_review_rows_child_budgets_stay_focused() {
    for (path, budget) in [
        (CODE_REVIEW_ROWS_GUARD_PATH, 100),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/delegation.rs", 80),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/row_ownership.rs", 90),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/root_and_children.rs", 90),
        (ROOT_PATHS_PATH, 100),
        (ROOT_STATUSES_PATH, 80),
        (ROOT_CHILD_ROWS_PATH, 140),
        (ROOT_SOURCE_BLOBS_PATH, 100),
        (ROOT_INVENTORY_GUARD_PATH, 100),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/export_chain.rs", 100),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/budgets.rs", 120),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/status_mirrors.rs", 130),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/status_mirrors/code_review_owner.rs", 70),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/status_mirrors/structure_guard_rows.rs", 130),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/status_mirrors/folder_backed.rs", 80),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows.rs", 90),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/budgets.rs", 50),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/child_split_status.rs", 80),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/delegation.rs", 30),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/row_children.rs", 90),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/row_data_status.rs", 40),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/status_mirrors.rs", 80),
        (CODE_REVIEW_ROWS_PATH, 90),
        (REVIEW_GUARD_ROWS_PATH, 170),
        (STRUCTURE_GUARD_ROWS_PATH, 80),
        (STRUCTURE_GUARD_ROOT_AND_CHILDREN_PATH, 80),
        ("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/code_review_findings.rs", 80),
        ("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/p0_robustness.rs", 80),
        ("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/plugin_importer_dx.rs", 80),
        ("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/p0_native_fixture.rs", 80),
        ("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/f8_child_owner.rs", 80),
        ("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/late_api_cleanup.rs", 80),
        (STRUCTURE_GUARD_STATUS_DOCS_PATH, 110),
        (STRUCTURE_GUARD_FOLDER_BACKED_SUMMARY_PATH, 150),
        (STRUCTURE_GUARD_TYPED_ERROR_PATH, 60),
        (STRUCTURE_GUARD_ROW_DATA_OWNER_PATH, 70),
        (TYPED_ERROR_STRUCTURE_ROWS_PATH, 120),
        (PLUGIN_IMPORTER_ROWS_PATH, 70),
        (PLUGIN_IMPORTER_REVIEW_GUARDS_PATH, 80),
        (PLUGIN_IMPORTER_STATUS_DOCS_PATH, 60),
        (PLUGIN_IMPORTER_SOURCE_INVENTORY_PATH, 70),
        (PLUGIN_IMPORTER_STRUCTURE_ASSERTIONS_PATH, 130),
        (PLUGIN_IMPORTER_ROW_DATA_OWNER_PATH, 40),
        ("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/top_level.rs", 80),
        ("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/folder_backed.rs", 80),
        ("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure_assertions.rs", 80),
        ("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status_docs.rs", 80),
        (ROW_DATA_OWNER_PATH, 70),
    ] {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below its child-owner budget of {budget} lines; got {line_count}"
        );
    }
}

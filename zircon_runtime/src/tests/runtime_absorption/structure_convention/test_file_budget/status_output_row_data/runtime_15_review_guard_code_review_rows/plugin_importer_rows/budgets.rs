use super::*;

#[test]
fn runtime_15_plugin_importer_rows_status_output_guard_children_line_budgets_are_current() {
    for (path, budget) in [
        (PLUGIN_IMPORTER_STATUS_OUTPUT_GUARD_PATH, 90),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/budgets.rs", 50),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/child_split_status.rs", 80),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/delegation.rs", 30),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/row_children.rs", 90),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/row_data_status.rs", 40),
        ("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/status_mirrors.rs", 80),
    ] {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below its child-owner budget of {budget} lines; got {line_count}"
        );
    }
    assert_contains_all(
        "plugin-importer status-output guard child source blob reaches every child",
        &plugin_importer_status_output_guard_child_source_blob(),
        &[
            "runtime_15_plugin_importer_rows_row_data_owner_is_child_backed",
            "assert_plugin_importer_row_data_children_are_current",
            "assert_plugin_importer_row_data_owner_status_row_is_current",
            "assert_plugin_importer_row_data_status_mirrors_are_current",
            "runtime_15_plugin_importer_status_output_guard_folder_backed_status_is_current",
        ],
    );
}

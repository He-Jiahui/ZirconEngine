use super::*;

#[test]
fn runtime_15_typed_error_structure_rows_guard_children_line_budgets_are_current() {
    for (path, budget) in [
        (TYPED_ERROR_STRUCTURE_ROWS_STATUS_OUTPUT_GUARD_PATH, 100),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/budgets.rs",
            50,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/delegation.rs",
            30,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/folder_backed.rs",
            60,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/row_groups.rs",
            50,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/status_doc_paths.rs",
            70,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_structure_rows/status_mirrors.rs",
            120,
        ),
    ] {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below its child-owner budget of {budget} lines; got {line_count}"
        );
    }
    assert_contains_all(
        "typed-error structure row-data guard child source blob reaches every child",
        &typed_error_structure_rows_guard_child_source_blob(),
        &[
            TYPED_ERROR_STRUCTURE_ROWS_ROW_DATA_GUARD_NAME,
            "assert_typed_error_structure_row_groups_are_child_backed",
            "assert_status_doc_paths_rows_are_child_backed",
            "assert_typed_error_structure_row_data_status_is_current",
            "assert_typed_error_structure_rows_guard_status_is_current",
        ],
    );
}

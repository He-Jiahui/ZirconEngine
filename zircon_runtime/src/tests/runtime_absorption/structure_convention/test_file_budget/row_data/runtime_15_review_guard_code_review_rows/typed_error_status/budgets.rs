use super::*;

// Keep this path/budget inventory compact; its line budget is itself under test.
#[rustfmt::skip]
#[test]
fn runtime_15_review_guard_typed_error_status_docs_guard_children_line_budgets_are_current() {
    for (path, budget) in [
        (TYPED_ERROR_STATUS_DOCS_GUARD_PATH, 100),
        ("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status/budgets.rs", 45),
        ("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status/delegation.rs", 20),
        ("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status/folder_backed.rs", 55),
        ("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status/paths.rs", 130),
        ("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status/row_routes.rs", 90),
        ("tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/typed_error_status/status_mirrors.rs", 115),
    ] {
        let line_count = read_runtime_src(path).lines().count();
        assert!(line_count < budget, "{path} should stay below its child-owner budget of {budget} lines; got {line_count}");
    }
    assert_contains_all("typed-error status-doc guard child source blob reaches every child", &typed_error_status_docs_guard_child_source_blob(), &["runtime_15_review_guard_typed_error_status_docs_row_data_is_folder_backed", "assert_typed_error_status_doc_row_routes_are_child_backed", "assert_typed_error_status_doc_row_data_status_is_current", "assert_typed_error_status_doc_guard_status_is_current"]);
}

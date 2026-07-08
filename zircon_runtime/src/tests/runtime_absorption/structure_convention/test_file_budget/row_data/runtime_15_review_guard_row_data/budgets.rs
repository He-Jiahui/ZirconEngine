use super::*;

#[path = "budgets/delegation.rs"]
mod delegation;
#[path = "budgets/folder_backed.rs"]
mod folder_backed;
#[path = "budgets/root_rows.rs"]
mod root_rows;
#[path = "budgets/status_support_rows.rs"]
mod status_support_rows;
#[path = "budgets/typed_error_rows.rs"]
mod typed_error_rows;

const REVIEW_GUARD_ROW_DATA_BUDGETS_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/budgets.rs";
const REVIEW_GUARD_ROW_DATA_BUDGET_CHILD_ROOT: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/budgets";
const REVIEW_GUARD_ROW_DATA_BUDGET_CHILDREN: &[&str] = &[
    "delegation",
    "folder_backed",
    "root_rows",
    "status_support_rows",
    "typed_error_rows",
];

#[test]
fn runtime_15_review_guard_row_data_child_budgets_stay_focused() {
    for module_name in REVIEW_GUARD_ROW_DATA_BUDGET_CHILDREN {
        let child_path = review_guard_row_data_budget_child_path(module_name);
        assert!(
            read_runtime_src(&child_path)
                .contains("assert_runtime_15_review_guard_row_data_budgets"),
            "{child_path} should own a focused Runtime 15 review row-data budget check"
        );
    }
}

fn assert_runtime_15_review_guard_row_data_budgets(paths: &[(&str, usize)]) {
    for (path, budget) in paths {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < *budget,
            "{path} should stay below its focused Runtime 15 review row-data budget of {budget}; got {line_count} lines"
        );
    }
}

fn review_guard_row_data_budget_child_path(module_name: &str) -> String {
    format!("{REVIEW_GUARD_ROW_DATA_BUDGET_CHILD_ROOT}/{module_name}.rs")
}

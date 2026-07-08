use super::*;

#[test]
fn runtime_15_review_guard_row_data_root_child_rows_guard_budgets_are_current() {
    for (path, budget) in [
        (ROOT_CHILD_ROWS_PATH, 40),
        (ROOT_CHILD_ROWS_TOP_LEVEL_CHILD_PATH, 70),
        (ROOT_CHILD_ROWS_DELEGATION_CHILD_PATH, 45),
        (ROOT_CHILD_ROWS_TYPED_ERROR_ROWS_CHILD_PATH, 70),
        (ROOT_CHILD_ROWS_AGGREGATION_CHILD_PATH, 70),
        (ROOT_CHILD_ROWS_SPLIT_LAYOUT_CHILD_PATH, 40),
        (ROOT_CHILD_ROWS_SPLIT_LAYOUT_ROUTE_MOUNTS_CHILD_PATH, 90),
        (ROOT_CHILD_ROWS_SPLIT_LAYOUT_STATUS_CURRENT_CHILD_PATH, 120),
        (ROOT_CHILD_ROWS_SPLIT_LAYOUT_BUDGETS_CHILD_PATH, 75),
        (ROOT_CHILD_ROWS_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD_PATH, 170),
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay under its focused root child rows budget of {budget}; got {line_count} lines"
        );
    }
}

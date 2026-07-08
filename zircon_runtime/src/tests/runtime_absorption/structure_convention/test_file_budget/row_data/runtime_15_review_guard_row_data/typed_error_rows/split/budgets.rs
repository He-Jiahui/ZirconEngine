use super::*;

#[test]
fn runtime_15_review_guard_typed_error_rows_guard_budgets_are_current() {
    for (path, budget) in [
        (TYPED_ERROR_ROWS_GUARD_PATH, 40),
        (TYPED_ERROR_ROWS_ROUTE_CHILDREN_PATH, 45),
        (TYPED_ERROR_ROWS_REPRESENTATIVE_ROWS_PATH, 45),
        (TYPED_ERROR_ROWS_EXPORT_CHAIN_PATH, 45),
        (TYPED_ERROR_ROWS_STATUS_MIRRORS_PATH, 95),
        (TYPED_ERROR_ROWS_SPLIT_LAYOUT_PATH, 40),
        (TYPED_ERROR_ROWS_SPLIT_LAYOUT_ROUTE_MOUNTS_CHILD_PATH, 90),
        (TYPED_ERROR_ROWS_SPLIT_LAYOUT_STATUS_CURRENT_CHILD_PATH, 120),
        (TYPED_ERROR_ROWS_SPLIT_LAYOUT_BUDGETS_CHILD_PATH, 75),
        (TYPED_ERROR_ROWS_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD_PATH, 170),
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay under its focused typed-error rows guard budget of {budget}; got {line_count} lines"
        );
    }
}

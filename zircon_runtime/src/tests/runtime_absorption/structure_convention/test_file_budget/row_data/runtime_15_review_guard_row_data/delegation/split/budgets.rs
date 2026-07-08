use super::*;

#[test]
fn runtime_15_review_guard_row_data_delegation_guard_budgets_are_current() {
    for (path, budget) in [
        (DELEGATION_GUARD_PATH, 35),
        (DELEGATION_ROUTE_MOUNTS_CHILD_PATH, 80),
        (DELEGATION_STATUS_INVENTORY_CHILD_PATH, 45),
        (DELEGATION_CHILD_INVENTORY_CHILD_PATH, 45),
        (DELEGATION_SPLIT_LAYOUT_CHILD_PATH, 40),
        (DELEGATION_SPLIT_LAYOUT_ROUTE_MOUNTS_CHILD_PATH, 90),
        (DELEGATION_SPLIT_LAYOUT_STATUS_CURRENT_CHILD_PATH, 120),
        (DELEGATION_SPLIT_LAYOUT_BUDGETS_CHILD_PATH, 75),
        (DELEGATION_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD_PATH, 170),
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay under its focused delegation guard budget of {budget}; got {line_count} lines"
        );
    }
}

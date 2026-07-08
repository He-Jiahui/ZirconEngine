use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_sources_stay_budgeted() {
    for (path, limit) in [
        (BUDGETS_SOURCES_PATH, 25usize),
        (BUDGETS_SOURCES_CHILDREN[0], 45),
        (BUDGETS_SOURCES_CHILDREN[1], 80),
        (BUDGETS_SOURCES_CHILDREN[2], 70),
        (BUDGETS_SOURCES_CHILDREN[3], 70),
        (BUDGETS_SOURCES_CHILDREN[4], 115),
        (BUDGETS_SOURCES_CHILDREN[5], 70),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the child-owner budget source inventory limit {limit}; got {line_count} lines"
        );
    }
}

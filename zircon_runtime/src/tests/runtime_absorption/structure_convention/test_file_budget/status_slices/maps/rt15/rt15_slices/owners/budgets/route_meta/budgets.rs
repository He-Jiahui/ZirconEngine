use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_route_metadata_children_stay_budgeted(
) {
    for (path, limit) in [
        (BUDGETS_ROUTE_PATH, 20usize),
        (BUDGETS_SOURCES_PATH, 180),
        (BUDGETS_GUARD_BODY_PATH, 30),
        (BUDGETS_ROUTE_METADATA_PATH, 25),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the child-owner budget route budget {limit}; got {line_count} lines"
        );
    }

    for (path, limit) in [
        (BUDGET_ROUTE_METADATA_CHILDREN[0], 50usize),
        (BUDGET_ROUTE_METADATA_CHILDREN[1], 95),
        (BUDGET_ROUTE_METADATA_CHILDREN[2], 70),
        (BUDGET_ROUTE_METADATA_CHILDREN[3], 60),
        (BUDGET_ROUTE_METADATA_CHILDREN[4], 80),
        (BUDGET_ROUTE_METADATA_CHILDREN[5], 90),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count <= limit,
            "{path} should stay below the child-owner budget route metadata budget {limit}; got {line_count} lines"
        );
    }
}

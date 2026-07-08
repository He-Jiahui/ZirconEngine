use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_parent_route_guard_body_route_metadata_children_stay_budgeted(
) {
    for (path, limit) in [
        (PARENT_ROUTE_GUARD_BODY_ROUTE_PATH, 20usize),
        (PARENT_ROUTE_GUARD_BODY_CHILDREN[0], 35),
        (PARENT_ROUTE_GUARD_BODY_CHILDREN[1], 70),
        (PARENT_ROUTE_GUARD_BODY_CHILDREN[2], 25),
        (PARENT_ROUTE_GUARD_BODY_CHILDREN[3], 65),
        (PARENT_ROUTE_GUARD_BODY_CHILDREN[4], 85),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the parent-route guard-body budget {limit}; got {line_count} lines"
        );
    }

    for (path, limit) in [
        (PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_CHILDREN[0], 70usize),
        (PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_CHILDREN[1], 90),
        (PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_CHILDREN[2], 70),
        (PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_CHILDREN[3], 55),
        (PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_CHILDREN[4], 80),
        (PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_CHILDREN[5], 80),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the parent-route guard-body route metadata budget {limit}; got {line_count} lines"
        );
    }
}

use super::*;

#[test]
fn runtime_15_naming_boundary_expected_slice_guard_body_route_metadata_children_stay_budgeted() {
    for (path, limit) in [
        (GUARD_BODY_ROUTE_PATH, 20usize),
        (GUARD_BODY_ROUTE_CHILDREN[0], 75),
        (GUARD_BODY_ROUTE_CHILDREN[1], 45),
        (GUARD_BODY_ROUTE_CHILDREN[2], 25),
        (GUARD_BODY_ROUTE_CHILDREN[3], 45),
        (GUARD_BODY_ROUTE_CHILDREN[4], 75),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the naming-boundary guard-body route budget {limit}; got {line_count} lines"
        );
    }

    for (path, limit) in [
        (ROUTE_METADATA_CHILDREN[0], 65usize),
        (ROUTE_METADATA_CHILDREN[1], 90),
        (ROUTE_METADATA_CHILDREN[2], 65),
        (ROUTE_METADATA_CHILDREN[3], 40),
        (ROUTE_METADATA_CHILDREN[4], 85),
        (ROUTE_METADATA_CHILDREN[5], 75),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count <= limit,
            "{path} should stay below the naming-boundary route-metadata budget {limit}; got {line_count} lines"
        );
    }
}

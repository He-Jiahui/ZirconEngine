use super::*;

#[test]
fn runtime_15_naming_boundary_expected_slice_route_metadata_children_stay_budgeted() {
    for (path, limit) in [
        (STRUCTURE_NAMING_BOUNDARY_GUARD, 25usize),
        (STRUCTURE_NAMING_BOUNDARY_GUARD_CHILDREN[0], 115),
        (STRUCTURE_NAMING_BOUNDARY_GUARD_CHILDREN[1], 20),
        (STRUCTURE_NAMING_BOUNDARY_GUARD_CHILDREN[2], 25),
    ] {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < limit,
            "{path} should stay below the naming-boundary route budget {limit}; got {line_count} lines"
        );
    }

    for (path, limit) in [
        (ROUTE_METADATA_CHILDREN[0], 45usize),
        (ROUTE_METADATA_CHILDREN[1], 90),
        (ROUTE_METADATA_CHILDREN[2], 65),
        (ROUTE_METADATA_CHILDREN[3], 60),
        (ROUTE_METADATA_CHILDREN[4], 85),
        (ROUTE_METADATA_CHILDREN[5], 85),
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

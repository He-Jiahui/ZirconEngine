use super::*;

const STRUCTURE_SUPPORT_PARENT_ROUTE_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/paths.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/sources.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/guard_body.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/route_metadata.rs",
];

#[test]
fn runtime_15_structure_support_expected_slice_parent_route_metadata_sources_stay_budgeted() {
    for (path, limit) in [
        (STRUCTURE_SUPPORT_PARENT_ROUTE_CHILD, 35usize),
        (STRUCTURE_SUPPORT_PARENT_ROUTE_CHILDREN[0], 85),
        (STRUCTURE_SUPPORT_PARENT_ROUTE_CHILDREN[1], 20),
        (STRUCTURE_SUPPORT_PARENT_ROUTE_CHILDREN[2], 20),
        (STRUCTURE_SUPPORT_PARENT_ROUTE_CHILDREN[3], 25),
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < limit,
            "{path} should stay below the parent-route metadata child budget {limit}; got {line_count} lines"
        );
    }

    for (path, limit) in [
        (PARENT_ROUTE_METADATA_CHILDREN[0], 60usize),
        (PARENT_ROUTE_METADATA_CHILDREN[1], 105),
        (PARENT_ROUTE_METADATA_CHILDREN[2], 75),
        (PARENT_ROUTE_METADATA_CHILDREN[3], 105),
        (PARENT_ROUTE_METADATA_CHILDREN[4], 95),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the parent-route route-metadata budget {limit}; got {line_count} lines"
        );
    }
}

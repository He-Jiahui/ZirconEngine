use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_route_metadata_children_stay_budgeted(
) {
    for (path, limit) in [
        (SPLIT_LAYOUT_PATH, 20usize),
        (SPLIT_LAYOUT_SOURCES_PATH, 110),
        (SPLIT_LAYOUT_GUARD_BODY_PATH, 160),
        (SPLIT_LAYOUT_ROUTE_METADATA_PATH, 25),
        (SPLIT_LAYOUT_STATUS_MIRRORS_PATH, 100),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the child-owner split-layout route budget {limit}; got {line_count} lines"
        );
    }

    for (path, limit) in [
        (ROUTE_METADATA_CHILDREN[0], 45usize),
        (ROUTE_METADATA_CHILDREN[1], 95),
        (ROUTE_METADATA_CHILDREN[2], 70),
        (ROUTE_METADATA_CHILDREN[3], 60),
        (ROUTE_METADATA_CHILDREN[4], 85),
        (ROUTE_METADATA_CHILDREN[5], 90),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count <= limit,
            "{path} should stay below the child-owner split-layout route metadata budget {limit}; got {line_count} lines"
        );
    }
}

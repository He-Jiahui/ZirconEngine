use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_route_metadata_parent_routes_stay_budgeted() {
    for (path, limit) in [
        (STRUCTURE_REVIEW_CHILD_ROUTE_PARENT, 25usize),
        (STRUCTURE_REVIEW_CHILD_ROUTE_CHILDREN[0], 30),
        (STRUCTURE_REVIEW_CHILD_ROUTE_CHILDREN[1], 20),
        (STRUCTURE_REVIEW_CHILD_ROUTE_CHILDREN[2], 25),
        (STRUCTURE_REVIEW_CHILD_ROUTE_METADATA_CHILDREN[0], 30),
        (STRUCTURE_REVIEW_CHILD_ROUTE_METADATA_CHILDREN[1], 20),
        (STRUCTURE_REVIEW_CHILD_ROUTE_METADATA_CHILDREN[2], 20),
    ] {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < limit,
            "{path} should stay below the review-route route-metadata parent budget {limit}; got {line_count} lines"
        );
    }
}

use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_route_metadata_source_children_stay_budgeted() {
    for (path, limit) in [
        (REVIEW_ROUTE_CHILD_SOURCE_CHILDREN[0], 65usize),
        (REVIEW_ROUTE_CHILD_SOURCE_CHILDREN[1], 30),
        (REVIEW_ROUTE_CHILD_SOURCE_CHILDREN[2], 30),
        (REVIEW_ROUTE_CHILD_SOURCE_CHILDREN[3], 130),
        (REVIEW_ROUTE_CHILD_SOURCE_CHILDREN[4], 90),
        (REVIEW_ROUTE_CHILD_SOURCE_CHILDREN[5], 45),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the review-route sources child budget {limit}; got {line_count} lines"
        );
    }
}

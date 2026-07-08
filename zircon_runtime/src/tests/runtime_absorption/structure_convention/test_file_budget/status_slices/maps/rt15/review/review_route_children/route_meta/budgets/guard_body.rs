use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_route_metadata_guard_body_children_stay_budgeted() {
    for (path, limit) in [
        (REVIEW_ROUTE_GUARD_BODY_CHILDREN[0], 75usize),
        (REVIEW_ROUTE_GUARD_BODY_CHILDREN[1], 65),
        (REVIEW_ROUTE_GUARD_BODY_CHILDREN[2], 110),
        (REVIEW_ROUTE_GUARD_BODY_CHILDREN[3], 95),
        (REVIEW_ROUTE_GUARD_BODY_CHILDREN[4], 95),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the review-route guard-body child budget {limit}; got {line_count} lines"
        );
    }
}

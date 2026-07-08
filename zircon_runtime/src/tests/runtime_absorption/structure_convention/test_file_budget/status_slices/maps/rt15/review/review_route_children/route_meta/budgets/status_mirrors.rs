use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_route_metadata_status_mirrors_stay_budgeted() {
    for (path, limit) in [
        (REVIEW_ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[0], 65usize),
        (REVIEW_ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[1], 75),
        (REVIEW_ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[2], 95),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the review-route route-metadata status mirror budget {limit}; got {line_count} lines"
        );
    }
}

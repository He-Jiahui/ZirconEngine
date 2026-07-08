use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_route_metadata_route_mounts_stay_budgeted() {
    for (path, limit) in [
        (REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_CHILDREN[0], 65usize),
        (REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_CHILDREN[1], 20),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the review-route route-metadata route-mounts budget {limit}; got {line_count} lines"
        );
    }
    for (path, limit) in [
        (
            REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_CHILDREN[0],
            75usize,
        ),
        (
            REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_CHILDREN[1],
            70,
        ),
        (
            REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_CHILDREN[2],
            75,
        ),
        (
            REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_CHILDREN[3],
            105,
        ),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the review-route route-metadata route-mounts folder-backed child budget {limit}; got {line_count} lines"
        );
    }
}

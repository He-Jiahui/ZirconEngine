use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_root_route_metadata_sources_stay_budgeted() {
    for (path, limit) in [
        (STRUCTURE_REVIEW_GUARD_PARENT, 30usize),
        (STRUCTURE_REVIEW_GUARD_SOURCES, 170),
        (STRUCTURE_REVIEW_GUARD_BODY, 20),
        (STRUCTURE_REVIEW_GUARD_ROUTE_METADATA, 25),
        (STRUCTURE_REVIEW_ROUTE_METADATA_CHILDREN[1], 20),
        (STRUCTURE_REVIEW_ROUTE_METADATA_CHILDREN[2], 20),
    ] {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < limit,
            "{path} should stay below the review-guard root route budget {limit}; got {line_count} lines"
        );
    }

    for (path, limit) in [
        (STRUCTURE_REVIEW_ROUTE_METADATA_CHILDREN[0], 55usize),
        (STRUCTURE_REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_CHILDREN[0], 65),
        (
            STRUCTURE_REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_CHILDREN[1],
            105,
        ),
    ] {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < limit,
            "{path} should stay below the review-guard route-metadata child budget {limit}; got {line_count} lines"
        );
    }

    for path in STRUCTURE_REVIEW_ROUTE_METADATA_STATUS_MIRROR_CHILDREN {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < 95,
            "{path} should stay below the review-guard route-metadata status-mirror child budget; got {line_count} lines"
        );
    }
}

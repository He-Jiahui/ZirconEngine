use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_structure_guard_body_sources_stay_budgeted() {
    for (path, limit) in [
        (STRUCTURE_REVIEW_GUARD_PARENT, 30usize),
        (STRUCTURE_REVIEW_GUARD_SOURCES, 170),
        (STRUCTURE_REVIEW_GUARD_BODY, 20),
        (STRUCTURE_REVIEW_GUARD_ROUTE_METADATA, 175),
        (STRUCTURE_REVIEW_STRUCTURE_SUPPORT_GUARD_CHILD, 400),
        (STRUCTURE_REVIEW_TYPED_ERROR_GUARD_CHILD, 400),
        (STRUCTURE_REVIEW_STATUS_SUPPORT_GUARD_CHILD, 400),
        (STRUCTURE_REVIEW_ROUTE_CHILD, 400),
    ] {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < limit,
            "{path} should stay below the Runtime 15 structure guard budget {limit}; got {line_count} lines"
        );
    }

    for (path, limit) in [
        (STRUCTURE_REVIEW_GUARD_BODY_CHILDREN[0], 75usize),
        (STRUCTURE_REVIEW_GUARD_BODY_CHILDREN[1], 25),
        (STRUCTURE_REVIEW_GUARD_BODY_CHILDREN[2], 95),
        (STRUCTURE_REVIEW_GUARD_BODY_CHILDREN[3], 25),
        (STRUCTURE_REVIEW_GUARD_BODY_CHILDREN[4], 75),
    ] {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < limit,
            "{path} should stay below the Runtime 15 structure guard-body child budget {limit}; got {line_count} lines"
        );
    }
}

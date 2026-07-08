use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_typed_error_route_metadata_children_stay_budgeted() {
    for (path, limit) in [
        (STRUCTURE_TYPED_ERROR_EXPECTED_SLICE_GUARD, 25usize),
        (STRUCTURE_TYPED_ERROR_EXPECTED_SLICE_GUARD_CHILDREN[0], 35),
        (STRUCTURE_TYPED_ERROR_EXPECTED_SLICE_GUARD_CHILDREN[1], 95),
        (STRUCTURE_TYPED_ERROR_EXPECTED_SLICE_GUARD_CHILDREN[2], 180),
        (STRUCTURE_TYPED_ERROR_EXPECTED_SLICE_GUARD_CHILDREN[3], 25),
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < limit,
            "{path} should stay below the typed-error expected-slice route budget {limit}; got {line_count} lines"
        );
    }

    for (path, limit) in [
        (ROUTE_METADATA_CHILDREN[0], 55usize),
        (ROUTE_METADATA_CHILDREN[1], 90),
        (ROUTE_METADATA_CHILDREN[2], 70),
        (ROUTE_METADATA_CHILDREN[3], 60),
        (ROUTE_METADATA_CHILDREN[4], 80),
        (ROUTE_METADATA_CHILDREN[5], 90),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the typed-error route-metadata child budget {limit}; got {line_count} lines"
        );
    }

    for (path, limit) in [
        (STRUCTURE_REVIEW_TYPED_ERROR_GUARD_BODY_CHILDREN[0], 70usize),
        (STRUCTURE_REVIEW_TYPED_ERROR_GUARD_BODY_CHILDREN[1], 80),
        (STRUCTURE_REVIEW_TYPED_ERROR_GUARD_BODY_CHILDREN[2], 45),
        (STRUCTURE_REVIEW_TYPED_ERROR_GUARD_BODY_CHILDREN[3], 45),
        (STRUCTURE_REVIEW_TYPED_ERROR_GUARD_BODY_CHILDREN[4], 120),
    ] {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < limit,
            "{path} should stay below the typed-error guard-body child budget {limit}; got {line_count} lines"
        );
    }
}

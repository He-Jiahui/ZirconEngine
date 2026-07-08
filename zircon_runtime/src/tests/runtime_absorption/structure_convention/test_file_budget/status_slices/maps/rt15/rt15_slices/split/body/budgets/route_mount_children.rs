use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_route_mounts_budgeted() {
    for (path, limit) in [
        (GUARD_BODY_ROUTE_MOUNTS_CHILDREN[0], 60usize),
        (GUARD_BODY_ROUTE_MOUNTS_CHILDREN[1], 45),
        (GUARD_BODY_ROUTE_MOUNTS_CHILDREN[2], 70),
        (GUARD_BODY_ROUTE_MOUNTS_CHILDREN[3], 75),
        (GUARD_BODY_ROUTE_MOUNTS_CHILDREN[4], 45),
        (GUARD_BODY_ROUTE_MOUNTS_CHILDREN[5], 50),
        (GUARD_BODY_ROUTE_MOUNTS_CHILDREN[6], 95),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count <= limit,
            "{path} should stay below the expected-slice maps guard-body route-mount budget {limit}; got {line_count} lines"
        );
    }
}

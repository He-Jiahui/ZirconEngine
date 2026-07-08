use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_children_stay_budgeted() {
    for (path, limit) in [
        (GUARD_BODY_CHILDREN[0], 25usize),
        (GUARD_BODY_CHILDREN[1], 90),
        (GUARD_BODY_CHILDREN[2], 75),
        (GUARD_BODY_CHILDREN[3], 70),
        (GUARD_BODY_CHILDREN[4], 25),
        (GUARD_BODY_CHILDREN[5], 75),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count <= limit,
            "{path} should stay below the expected-slice maps guard-body budget {limit}; got {line_count} lines"
        );
    }
}

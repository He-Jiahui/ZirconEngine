use super::*;

#[test]
fn runtime_15_status_support_m3_m4_expected_slice_maps_guard_children_stay_budgeted() {
    for (path, limit) in [
        (STRUCTURE_GUARD_PATH, 25usize),
        (GUARD_CHILDREN[0], 45),
        (GUARD_CHILDREN[1], 75),
        (GUARD_CHILDREN[2], 95),
        (GUARD_CHILDREN[3], 105),
        (GUARD_CHILDREN[4], 75),
        (GUARD_CHILDREN[5], 95),
    ] {
        let runtime_path = if path.starts_with("tests/") {
            path.to_string()
        } else {
            format!("tests/runtime_absorption/{path}")
        };
        let line_count = read_runtime_src(&runtime_path).lines().count();
        assert!(
            line_count <= limit,
            "{path} should stay below the M3/M4 map guard budget {limit}; got {line_count} lines"
        );
    }
}

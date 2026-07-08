use super::*;

#[test]
fn runtime_15_foundation_expected_slice_maps_guard_children_stay_budgeted() {
    for (path, max_lines) in [
        (FOUNDATION_ROUTE_PARENT, 30usize),
        (FOUNDATION_GUARD_CHILDREN[0], 55),
        (FOUNDATION_GUARD_CHILDREN[1], 80),
        (FOUNDATION_GUARD_CHILDREN[2], 70),
        (FOUNDATION_GUARD_CHILDREN[3], 95),
        (FOUNDATION_GUARD_CHILDREN[4], 75),
        (FOUNDATION_GUARD_CHILDREN[5], 20),
        (FOUNDATION_STATUS_MIRRORS_CHILDREN[0], 35),
        (FOUNDATION_STATUS_MIRRORS_CHILDREN[1], 85),
        (FOUNDATION_STATUS_MIRRORS_CHILDREN[2], 45),
        (FOUNDATION_STATUS_MIRRORS_CHILDREN[3], 20),
        (FOUNDATION_STATUS_MIRRORS_CHILDREN[4], 80),
    ] {
        let line_count = if path == FOUNDATION_ROUTE_PARENT {
            read_runtime_src(path)
        } else {
            read_runtime_absorption_child(path)
        }
        .lines()
        .count();
        assert!(
            line_count <= max_lines,
            "{path} should stay below the foundation expected-slice guard budget {max_lines}; got {line_count} lines"
        );
    }
}

use super::*;

#[test]
fn runtime_15_foundation_expected_slice_maps_status_mirror_children_stay_budgeted() {
    for (path, max_lines) in [
        (FOUNDATION_STATUS_MIRRORS_PARENT, 20usize),
        (FOUNDATION_STATUS_MIRRORS_CHILDREN[0], 35),
        (FOUNDATION_STATUS_MIRRORS_CHILDREN[1], 85),
        (FOUNDATION_STATUS_MIRRORS_CHILDREN[2], 45),
        (FOUNDATION_STATUS_MIRRORS_CHILDREN[3], 20),
        (FOUNDATION_STATUS_MIRRORS_CHILDREN[4], 80),
    ] {
        let line_count = read_runtime_absorption_child(path).lines().count();
        assert!(
            line_count <= max_lines,
            "{path} should stay below the foundation status mirror budget {max_lines}; got {line_count} lines"
        );
    }
}

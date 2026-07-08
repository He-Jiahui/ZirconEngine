use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_route_children_stay_within_budget() {
    let parent_line_count = read_runtime_src(STRUCTURE_SUPPORT_EXPECTED_SLICE_PARENT)
        .lines()
        .count();
    assert!(
        parent_line_count < 60,
        "{STRUCTURE_SUPPORT_EXPECTED_SLICE_PARENT} should stay a route owner; got {parent_line_count} lines"
    );

    for child_path in STRUCTURE_SUPPORT_EXPECTED_SLICE_CHILDREN {
        let line_count = read_runtime_src(child_path).lines().count();
        assert!(
            line_count < 240,
            "{child_path} should stay below the Runtime 15 structure-support guard child budget; got {line_count} lines"
        );
    }
}

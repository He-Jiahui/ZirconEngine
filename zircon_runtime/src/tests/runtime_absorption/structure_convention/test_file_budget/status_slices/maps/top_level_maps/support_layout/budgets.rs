use super::*;

const TOP_LEVEL_SUPPORT_LAYOUT_PARENT: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/support_layout.rs";
const TOP_LEVEL_SUPPORT_LAYOUT_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/support_layout/parent_mounts.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/support_layout/assertion_helpers.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/support_layout/sources.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/support_layout/budgets.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/support_layout/status_mirrors.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/support_layout/split_layout.rs",
];

#[test]
fn runtime_15_top_level_expected_slice_support_layout_sources_stay_within_budget() {
    let owners = [
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps.rs",
        TOP_LEVEL_SUPPORT_LAYOUT_PARENT,
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/line_budgets.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/pre_runtime_15_maps.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/runtime_15_maps.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/status_and_docs.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/sources.rs",
    ];

    for path in owners
        .iter()
        .chain(TOP_LEVEL_SUPPORT_LAYOUT_CHILDREN.iter())
    {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the Runtime 15 focused guard budget; got {line_count} lines"
        );
    }

    let parent_line_count = read_runtime_src(TOP_LEVEL_SUPPORT_LAYOUT_PARENT)
        .lines()
        .count();
    assert!(
        parent_line_count < 80,
        "{TOP_LEVEL_SUPPORT_LAYOUT_PARENT} should stay a route owner; got {parent_line_count} lines"
    );
}

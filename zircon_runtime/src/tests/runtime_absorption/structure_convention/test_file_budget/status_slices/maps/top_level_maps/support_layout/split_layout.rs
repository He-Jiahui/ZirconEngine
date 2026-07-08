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

fn assert_top_level_support_layout_guard_is_folder_backed() {
    let parent = read_runtime_src(TOP_LEVEL_SUPPORT_LAYOUT_PARENT);
    let children = TOP_LEVEL_SUPPORT_LAYOUT_CHILDREN
        .iter()
        .map(|path| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "support-layout parent mounts focused children",
        &parent,
        &[
            "#[path = \"support_layout/parent_mounts.rs\"]",
            "mod parent_mounts;",
            "#[path = \"support_layout/assertion_helpers.rs\"]",
            "mod assertion_helpers;",
            "#[path = \"support_layout/sources.rs\"]",
            "mod sources;",
            "#[path = \"support_layout/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"support_layout/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"support_layout/split_layout.rs\"]",
            "mod split_layout;",
        ],
    );
    for moved_anchor in [
        "fn runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed",
        "top-level expected-slice map parent mounts support owners",
        "top-level map assertions child mounts focused assertion helpers",
        "status-output M3 row data",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "top_level_maps/support_layout.rs should delegate {moved_anchor}"
        );
    }
    assert_contains_all(
        "support-layout children own focused checks",
        &children,
        &[
            "runtime_15_top_level_expected_slice_support_layout_parent_mounts_are_child_owned",
            "runtime_15_top_level_expected_slice_assertion_helpers_are_child_owned",
            "runtime_15_top_level_expected_slice_sources_are_child_owned",
            "runtime_15_top_level_expected_slice_support_layout_sources_stay_within_budget",
            "runtime_15_top_level_expected_slice_support_layout_status_mirrors_are_current",
            "runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed",
            "runtime_15_top_level_expected_slice_support_layout_guard_is_folder_backed",
            "Runtime 15 M3 top-level expected-slice support-layout guard folder-backed split",
            "runtime_15_top_level_expected_slice_support_layout_guard_folder_backed_static_passed_cargo_deferred",
        ],
    );
}

#[test]
fn runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed() {
    assert_top_level_support_layout_guard_is_folder_backed();
}

#[test]
fn runtime_15_top_level_expected_slice_support_layout_guard_is_folder_backed() {
    assert_top_level_support_layout_guard_is_folder_backed();
}

use super::*;

const STRUCTURE_SUPPORT_EXPECTED_SLICE_PARENT: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure_support_expected_slice.rs";
const STRUCTURE_SUPPORT_EXPECTED_SLICE_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_mounts.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/literal_ownership.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/budgets.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/status_mirrors.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/split_layout.rs",
];
const STRUCTURE_SUPPORT_STATUS_MIRROR_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/mirrors/folder_backed.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/mirrors/row_maps.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/mirrors/status_docs.rs",
];

fn assert_structure_support_guard_is_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_SUPPORT_EXPECTED_SLICE_PARENT);
    let direct_children = STRUCTURE_SUPPORT_EXPECTED_SLICE_CHILDREN
        .iter()
        .map(|path| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n");
    let status_mirror_children = STRUCTURE_SUPPORT_STATUS_MIRROR_CHILDREN
        .iter()
        .map(|path| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n");
    let children = format!("{direct_children}\n{status_mirror_children}");

    assert_contains_all(
        "structure-support expected-slice guard parent mounts children",
        &parent,
        &[
            "#[path = \"structure/parent_mounts.rs\"]",
            "mod parent_mounts;",
            "#[path = \"structure/literal_ownership.rs\"]",
            "mod literal_ownership;",
            "#[path = \"structure/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"structure/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"structure/split_layout.rs\"]",
            "mod split_layout;",
            "#[path = \"structure/parent_route_children.rs\"]",
            "mod parent_route_children;",
        ],
    );
    for moved_anchor in [
        "fn runtime_15_structure_support_expected_slice_maps_are_child_owners",
        "M3 structure-support status expected-slice parent mounts map children",
        "review expected-slice children own review guard literals",
        "status-support expected-slice children own status-support literals",
        "status-output M3 row data",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "structure_support_expected_slice.rs should delegate {moved_anchor}"
        );
    }
    assert_contains_all(
        "structure-support expected-slice guard children own split checks",
        &children,
        &[
            "runtime_15_structure_support_expected_slice_parent_mounts_are_child_owned",
            "runtime_15_structure_support_expected_slice_literals_are_child_owned",
            "runtime_15_structure_support_expected_slice_sources_stay_within_budget",
            "runtime_15_structure_support_expected_slice_status_mirrors_are_folder_backed",
            "runtime_15_structure_support_expected_slice_status_rows_are_synced",
            "runtime_15_structure_support_expected_slice_guard_is_folder_backed",
            "Runtime 15 M3 structure-support expected-slice guard folder-backed split",
            "runtime_15_structure_support_expected_slice_guard_folder_backed_static_passed_cargo_deferred",
        ],
    );
}

#[test]
fn runtime_15_structure_support_expected_slice_maps_are_child_owners() {
    assert_structure_support_guard_is_folder_backed();
}

#[test]
fn runtime_15_structure_support_expected_slice_guard_is_folder_backed() {
    assert_structure_support_guard_is_folder_backed();
}

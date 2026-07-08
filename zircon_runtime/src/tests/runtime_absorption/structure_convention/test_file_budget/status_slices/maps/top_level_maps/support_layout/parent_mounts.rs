use super::*;

#[test]
fn runtime_15_top_level_expected_slice_support_layout_parent_mounts_are_child_owned() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps.rs",
    );

    assert_contains_all(
        "top-level expected-slice map parent mounts support owners",
        &parent,
        &[
            "#[path = \"top_level_maps/assertions.rs\"]",
            "mod assertions;",
            "#[path = \"top_level_maps/sources.rs\"]",
            "mod sources;",
            "#[path = \"top_level_maps/support_layout.rs\"]",
            "mod support_layout;",
            "fn runtime_15_status_output_expected_slice_maps_are_child_owners",
        ],
    );
    for moved_anchor in [
        concat!("pub(super) struct ", "TopLevelMapSources"),
        concat!(
            "pub(super) fn assert_expected_slice_maps_",
            "are_child_owners"
        ),
        concat!("let status_parent = ", "read_runtime_src("),
        concat!(
            "Runtime 15 status expected-slice child ",
            "delegates topic owners"
        ),
        "fn runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed",
        "Runtime 15 M3 top-level expected-slice support-layout guard body child split",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "maps/top_level_maps.rs should mount support child owners instead of keeping {moved_anchor}"
        );
    }
}

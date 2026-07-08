use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_is_folder_backed() {
    let parent = read_runtime_15_map_parent();

    assert_contains_all(
        "Runtime 15 expected-slice map guard parent mounts children",
        &parent,
        &[
            "mod child_owners;",
            "mod naming_boundary;",
            "mod split_layout;",
        ],
    );
    for moved_guard in [
        "fn runtime_15_status_output_runtime_15_expected_slice_maps_are_child_owners",
        "fn runtime_15_status_output_naming_boundary_expected_slice_maps_are_folder_backed",
        "fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_is_folder_backed",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "Runtime 15 expected-slice maps parent should delegate `{moved_guard}`"
        );
    }
}

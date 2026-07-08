use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_is_folder_backed() {
    let parent = read_child_owner_parent();

    assert_contains_all(
        "Runtime 15 expected-slice child-owner guard parent mounts children",
        &parent,
        &[
            "mod budgets;",
            "mod literal_ownership;",
            "mod route_mounts;",
            "mod split_layout;",
            "mod status_mirrors;",
        ],
    );
    for moved_guard in [
        "fn runtime_15_status_output_runtime_15_expected_slice_maps_are_child_owners",
        "fn runtime_15_expected_slice_child_literals_stay_child_owned",
        "fn runtime_15_expected_slice_child_owner_sources_stay_budgeted",
        "fn runtime_15_expected_slice_child_owner_status_mirrors_stay_synced",
        "fn runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_is_folder_backed",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "Runtime 15 expected-slice child-owner parent should delegate `{moved_guard}`"
        );
    }
}

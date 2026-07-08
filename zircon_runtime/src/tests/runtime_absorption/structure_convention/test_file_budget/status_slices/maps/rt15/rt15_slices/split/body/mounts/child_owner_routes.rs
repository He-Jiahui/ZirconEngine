use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_child_owner_routes_are_child_owned(
) {
    let child_owners = read_runtime_15_map("child_owners.rs");

    assert_contains_all(
        "Runtime 15 expected-slice child-owner guard child",
        &child_owners,
        &[
            "mod budgets;",
            "mod literal_ownership;",
            "mod route_mounts;",
            "mod split_layout;",
            "mod status_mirrors;",
        ],
    );
}

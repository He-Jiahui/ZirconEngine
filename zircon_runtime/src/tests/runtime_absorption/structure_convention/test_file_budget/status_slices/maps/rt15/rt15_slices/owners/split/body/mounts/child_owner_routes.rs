use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_child_routes_are_child_owned(
) {
    for (label, source, guard_name) in [
        (
            "Runtime 15 expected-slice route-mount child",
            read_child_owner("route_mounts.rs"),
            "fn runtime_15_status_output_runtime_15_expected_slice_maps_are_child_owners",
        ),
        (
            "Runtime 15 expected-slice literal ownership status-literals child",
            read_child_owner("literal/status_literals.rs"),
            "fn runtime_15_expected_slice_child_literals_stay_child_owned",
        ),
        (
            "Runtime 15 expected-slice status mirrors child",
            read_child_owner("status_mirrors.rs"),
            "fn runtime_15_expected_slice_child_owner_status_mirrors_stay_synced",
        ),
    ] {
        assert_contains_all(label, &source, &[guard_name]);
    }
}

use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_maps_are_child_owners() {
    let status_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
    );
    let date_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
    );

    assert_contains_all(
        "Runtime 15 status expected-slice parent mounts topic owners",
        &status_runtime_15,
        &[
            "mod foundation;",
            "mod naming_boundary;",
            "mod m4_surface_cleanup;",
            "mod m3_structure_support;",
            "foundation::expected_status_for_slice(slice)",
            "m3_structure_support::expected_status_for_slice(slice)",
        ],
    );
    assert_contains_all(
        "Runtime 15 date expected-slice parent mounts topic owners",
        &date_runtime_15,
        &[
            "mod foundation;",
            "mod naming_boundary;",
            "mod m4_surface_cleanup;",
            "mod m3_structure_support;",
            "foundation::expected_date_for_slice(slice)",
            "m3_structure_support::expected_date_for_slice(slice)",
        ],
    );
    for moved_literal in [
        "Runtime 15 M3 lock poison policy guard folder split",
        "Runtime 15 M2 core runtime state module naming hard cutover",
        "Runtime 15 M4 scene world project I/O mesh owner split",
        "Runtime 15 M3 status output expected-slice guard child-owner split",
    ] {
        assert!(
            !status_runtime_15.contains(moved_literal),
            "Runtime 15 status expected-slice parent should delegate {moved_literal}"
        );
        assert!(
            !date_runtime_15.contains(moved_literal),
            "Runtime 15 date expected-slice parent should delegate {moved_literal}"
        );
    }
}

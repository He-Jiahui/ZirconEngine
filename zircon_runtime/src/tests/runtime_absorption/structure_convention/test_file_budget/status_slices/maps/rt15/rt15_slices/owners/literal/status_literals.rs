use super::*;

#[test]
fn runtime_15_expected_slice_child_literals_stay_child_owned() {
    let status_child_sources = joined_runtime_sources(STATUS_CHILD_SOURCE_PATHS);

    assert_contains_all(
        "Runtime 15 status expected-slice children own topic literals",
        &status_child_sources,
        &[
            "Runtime 15 M3 core runtime lock poison guard child-owner split",
            "Runtime 15 M2 core runtime state module naming hard cutover",
            "Runtime 15 M4 scene world project I/O mesh owner split",
            "Runtime 15 M3 status output Runtime 15 expected-slice child-owner split",
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_split_static_passed_cargo_deferred",
        ],
    );
}

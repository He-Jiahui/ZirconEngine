use super::*;

#[test]
fn runtime_15_expected_slice_child_date_literals_stay_child_owned() {
    let date_child_sources = joined_runtime_sources(DATE_CHILD_SOURCE_PATHS);

    assert_contains_all(
        "Runtime 15 date expected-slice children own topic literals",
        &date_child_sources,
        &[
            "Runtime 15 M3 core runtime lock poison guard child-owner split",
            "Runtime 15 M2 core runtime state module naming hard cutover",
            "Runtime 15 M4 scene world project I/O mesh owner split",
            "Runtime 15 M3 status output Runtime 15 expected-slice child-owner split",
            "Some(\"2026-06-25\")",
        ],
    );
}

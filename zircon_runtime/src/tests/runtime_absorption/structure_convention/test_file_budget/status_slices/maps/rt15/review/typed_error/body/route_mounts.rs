use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_typed_error_maps_are_child_owned() {
    let status_review_child = read_runtime_src(STATUS_REVIEW_CHILD);
    let date_review_child = read_runtime_src(DATE_REVIEW_CHILD);

    assert_contains_all(
        "review expected-slice parents mount typed-error children",
        &format!("{status_review_child}\n{date_review_child}"),
        &[
            "#[path = \"review/typed_error_maps.rs\"]",
            "mod typed_error_maps;",
            "typed_error_maps::expected_status_for_slice(slice)",
            "typed_error_maps::expected_date_for_slice(slice)",
        ],
    );
}

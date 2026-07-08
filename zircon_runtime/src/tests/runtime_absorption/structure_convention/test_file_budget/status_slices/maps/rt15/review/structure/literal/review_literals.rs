use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_review_literals_are_child_owned() {
    let status_review_child = read_runtime_src(STATUS_REVIEW_CHILD);
    let status_review_foundation_child = read_status_review_foundation_sources();
    let status_review_typed_error_child = read_status_review_typed_error_sources();
    let status_review_top_row_child = read_runtime_src(STATUS_REVIEW_TOP_ROW_CHILD);
    let date_review_child = read_runtime_src(DATE_REVIEW_CHILD);
    let date_review_foundation_child = read_date_review_foundation_sources();
    let date_review_typed_error_child = read_date_review_typed_error_sources();
    let date_review_top_row_child = read_runtime_src(DATE_REVIEW_TOP_ROW_CHILD);

    assert_contains_all(
        "review expected-slice children own review guard literals",
        &format!(
            "{status_review_child}\n{status_review_foundation_child}\n{status_review_typed_error_child}\n{status_review_top_row_child}\n{date_review_child}\n{date_review_foundation_child}\n{date_review_typed_error_child}\n{date_review_top_row_child}"
        ),
        &[
            "Runtime 15 M3 P0 robustness review guard child-owner split",
            "runtime_15_native_plugin_loader_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 review guard typed-error expected-slice map child split",
            "runtime_15_review_guard_typed_error_expected_slice_map_child_split_static_passed_cargo_deferred",
            "Runtime 15 M3 D12 runtime helper export macro review sync",
            "Some(\"2026-06-30\")",
        ],
    );
}

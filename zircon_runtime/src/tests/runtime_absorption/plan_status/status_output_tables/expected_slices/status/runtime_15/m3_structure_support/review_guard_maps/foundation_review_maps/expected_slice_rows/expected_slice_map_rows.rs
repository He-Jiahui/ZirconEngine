pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard foundation expected-slice rows folder-backed split" => Some(
            "runtime_15_review_guard_foundation_expected_slice_rows_folder_backed_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard code-review expected-slice map rows folder-backed split" => {
            Some("runtime_15_review_guard_code_review_expected_slice_map_rows_folder_backed_static_passed_cargo_deferred")
        }
        _ => None,
    }
}

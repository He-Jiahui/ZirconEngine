pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review guard typed-error expected-slice map child split" => Some(
            "runtime_15_review_guard_typed_error_expected_slice_map_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard typed-error expected-slice map rows folder-backed split" => Some(
            "runtime_15_review_guard_typed_error_expected_slice_map_rows_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard typed-error expected-slice route metadata child split" => {
            Some("runtime_15_review_guard_typed_error_expected_slice_route_metadata_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 review-guard typed-error expected-slice route metadata folder-backed split" => {
            Some("runtime_15_review_guard_typed_error_expected_slice_route_metadata_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 review-guard typed-error expected-slice guard body folder-backed split" => {
            Some("runtime_15_review_guard_typed_error_expected_slice_guard_body_folder_backed_static_passed_cargo_deferred")
        }
        _ => None,
    }
}

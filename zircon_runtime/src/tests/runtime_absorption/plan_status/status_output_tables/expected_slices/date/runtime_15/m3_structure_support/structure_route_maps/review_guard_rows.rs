pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard code-review expected-slice map rows folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 review-guard typed-error expected-slice map rows folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 typed-error status-doc expected-slice rows guard folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 review-guard foundation expected-slice rows guard folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 status-support runtime-index anchor expected-slice maps guard folder-backed split" => {
            Some("2026-07-07")
        }
        "Runtime 15 M3 structure-route status-date maps folder-backed split" => Some("2026-07-07"),
        _ => None,
    }
}

// Guard: runtime_15_review_guard_code_review_expected_slice_map_rows_are_folder_backed.
// Guard: runtime_15_review_guard_typed_error_expected_slice_map_rows_are_folder_backed.
// Guard: runtime_15_typed_error_status_doc_expected_slice_rows_guard_is_folder_backed.
// Guard: runtime_15_review_guard_foundation_expected_slice_rows_guard_is_folder_backed.
// Guard: runtime_15_status_support_runtime_index_anchor_expected_slice_maps_guard_is_folder_backed.
// Guard: runtime_15_structure_route_status_date_maps_are_folder_backed.

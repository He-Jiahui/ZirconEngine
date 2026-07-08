pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status-support review-guard row-data expected-slice maps folder-backed split" => Some(
            "runtime_15_status_support_review_guard_row_data_expected_slice_maps_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output review-guard row-data guard child-owner split" => Some(
            "runtime_15_status_output_review_guard_row_data_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data delegation guard folder-backed split" => Some(
            "runtime_15_review_guard_row_data_delegation_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data delegation split-layout guard folder-backed split" => Some(
            "runtime_15_review_guard_row_data_delegation_split_layout_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data root child rows folder-backed split" => Some(
            "runtime_15_review_guard_row_data_root_child_rows_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data root child rows split-layout guard folder-backed split" => Some(
            "runtime_15_review_guard_row_data_root_child_rows_split_layout_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data status-doc guard child-owner split" => Some(
            "runtime_15_review_guard_row_data_status_docs_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data moved-row guard child-owner split" => Some(
            "runtime_15_review_guard_row_data_moved_rows_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data budgets guard folder-backed split" => Some(
            "runtime_15_review_guard_row_data_budgets_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data root paths folder-backed split" => Some(
            "runtime_15_review_guard_row_data_root_paths_folder_backed_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

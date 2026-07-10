pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard row-data moved-row guard child-owner split" => Some(
            "runtime_15_review_guard_row_data_moved_rows_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard moved-row guard folder-backed split" => Some(
            "runtime_15_review_guard_moved_row_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard moved-row status-mirror child split" => Some(
            "runtime_15_review_guard_moved_row_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard moved-row root inventory child split" => Some(
            "runtime_15_review_guard_moved_row_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard moved-row code-review rows child split" => Some(
            "runtime_15_review_guard_moved_row_code_review_rows_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard moved-row code-review rows route metadata child split" => {
            Some(
                "runtime_15_review_guard_moved_row_code_review_rows_route_metadata_child_split_static_passed_cargo_deferred",
            )
        }
        _ => None,
    }
}

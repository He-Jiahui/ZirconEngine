pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard expected-slice root route metadata child split" => Some(
            "runtime_15_review_guard_expected_slice_root_route_metadata_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard expected-slice root guard body folder-backed split" => Some(
            "runtime_15_review_guard_expected_slice_root_guard_body_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard expected-slice root guard body child ownership folder-backed split" => Some(
            "runtime_15_review_guard_expected_slice_root_guard_body_child_ownership_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard expected-slice root guard body route mounts folder-backed split" => Some(
            "runtime_15_review_guard_expected_slice_root_guard_body_route_mounts_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard expected-slice root route metadata guard folder-backed split" => Some(
            "runtime_15_review_guard_expected_slice_root_route_metadata_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard expected-slice root route metadata route mounts folder-backed split" => Some(
            "runtime_15_review_guard_expected_slice_root_route_metadata_route_mounts_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard expected-slice root route metadata status-mirror guard folder-backed split" => Some(
            "runtime_15_review_guard_expected_slice_root_route_metadata_status_mirror_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard root source inventory folder-backed split" => Some(
            "runtime_15_review_guard_root_source_inventory_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard structure row data folder-backed split" => Some(
            "runtime_15_review_guard_structure_row_data_folder_backed_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

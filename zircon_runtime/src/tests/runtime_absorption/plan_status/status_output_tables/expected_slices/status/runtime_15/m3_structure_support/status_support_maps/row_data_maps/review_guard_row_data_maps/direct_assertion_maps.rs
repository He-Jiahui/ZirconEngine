pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard direct-assertion row-data guard folder-backed split" => Some(
            "runtime_15_review_guard_direct_assertion_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard direct-assertion status-mirror child split" => Some(
            "runtime_15_review_guard_direct_assertion_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard direct-assertion row-data root inventory child split" => Some(
            "runtime_15_review_guard_direct_assertion_row_data_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard direct-assertion row-data child-owner split" => Some(
            "runtime_15_review_guard_direct_assertion_row_data_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard direct-assertion row-data folder-backed split" => Some(
            "runtime_15_review_guard_direct_assertion_row_data_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard direct-assertion row-ownership guard child split" => Some(
            "runtime_15_review_guard_direct_assertion_row_ownership_guard_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard direct-assertion export-chain guard child split" => Some(
            "runtime_15_review_guard_direct_assertion_export_chain_guard_child_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

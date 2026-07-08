pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 code review findings source inventory child-owner split" => Some(
            "runtime_15_code_review_findings_source_inventory_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings source inventory folder-backed split" => Some(
            "runtime_15_code_review_findings_source_inventory_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings source inventory status-mirror child-owner split" => {
            Some("runtime_15_code_review_findings_source_inventory_status_mirror_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings source inventory map-source sync" => Some(
            "runtime_15_code_review_findings_source_inventory_map_source_sync_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

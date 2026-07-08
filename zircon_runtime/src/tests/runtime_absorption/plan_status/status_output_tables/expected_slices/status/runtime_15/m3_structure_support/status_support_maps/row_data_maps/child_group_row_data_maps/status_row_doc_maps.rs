pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 child-group status-row-doc guard child-owner split" => Some(
            "runtime_15_m3_child_group_status_row_docs_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-group status-row-doc guard folder-backed split" => Some(
            "runtime_15_m3_child_group_status_row_docs_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-group status-row-doc status-mirror child split" => Some(
            "runtime_15_m3_child_group_status_row_docs_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-group status-row-doc root inventory child split" => Some(
            "runtime_15_m3_child_group_status_row_docs_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-group status-row-doc source/status-map sync" => Some(
            "runtime_15_m3_child_group_status_row_docs_source_status_map_sync_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

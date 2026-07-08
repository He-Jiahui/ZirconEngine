pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status output Runtime 15 foundation row data split" => Some(
            "runtime_15_status_output_runtime_15_foundation_row_data_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data topic child-owner split" => Some(
            "runtime_15_foundation_row_data_topic_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data guard child-owner split" => Some(
            "runtime_15_foundation_row_data_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data status-doc guard child-owner split" => Some(
            "runtime_15_foundation_row_data_status_docs_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data 73-row docs sync" => Some(
            "runtime_15_foundation_row_data_71_row_docs_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data status-doc guard folder-backed split" => Some(
            "runtime_15_foundation_row_data_status_docs_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data row-count child split" => Some(
            "runtime_15_foundation_row_data_row_count_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data status-doc root inventory child split" => Some(
            "runtime_15_foundation_row_data_status_docs_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data stale-count prose guard" => Some(
            "runtime_15_foundation_row_data_stale_count_prose_guard_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data priority-doc frontmatter sync" => Some(
            "runtime_15_foundation_row_data_priority_doc_frontmatter_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data status-doc source/status-map sync" => Some(
            "runtime_15_foundation_row_data_status_docs_source_status_map_sync_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

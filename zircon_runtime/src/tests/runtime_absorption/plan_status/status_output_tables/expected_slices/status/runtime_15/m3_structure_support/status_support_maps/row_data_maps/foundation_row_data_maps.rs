pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 foundation row-data guard child-owner split" => Some(
            "runtime_15_foundation_row_data_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data guard folder-backed split" => Some(
            "runtime_15_foundation_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data status-mirror child split" => Some(
            "runtime_15_foundation_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data root inventory child split" => Some(
            "runtime_15_foundation_row_data_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data source/status-map sync" => Some(
            "runtime_15_foundation_row_data_source_status_map_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation-guards row-data guard folder-backed split" => Some(
            "runtime_15_foundation_guards_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation-guards row-data status-mirror child split" => Some(
            "runtime_15_foundation_guards_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation-guards row-data root inventory child split" => Some(
            "runtime_15_foundation_guards_row_data_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation-guards runtime-structure row-data child split" => Some(
            "runtime_15_foundation_guards_runtime_structure_row_data_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation-guards runtime-structure guard folder-backed split" => Some(
            "runtime_15_foundation_guards_runtime_structure_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data status-doc guard child-owner split" => Some(
            "runtime_15_foundation_row_data_status_docs_child_owner_split_static_passed_cargo_deferred",
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
        _ => None,
    }
}

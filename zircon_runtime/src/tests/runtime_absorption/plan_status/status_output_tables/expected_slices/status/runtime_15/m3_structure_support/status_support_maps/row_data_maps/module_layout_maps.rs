pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status-output row-data module-layout guard folder-backed split" => Some(
            "runtime_15_status_output_row_data_module_layout_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout status-mirror child split" => Some(
            "runtime_15_module_layout_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout root inventory child split" => Some(
            "runtime_15_module_layout_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout source/status-map sync" => Some(
            "runtime_15_module_layout_source_status_map_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output row-data module-layout status-doc guard child-owner split" => Some(
            "runtime_15_status_output_row_data_module_layout_status_docs_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout status-doc guard folder-backed split" => Some(
            "runtime_15_module_layout_status_docs_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout status-doc status-mirror child split" => Some(
            "runtime_15_module_layout_status_docs_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout status-doc root inventory child split" => Some(
            "runtime_15_module_layout_status_docs_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout status-doc source/status-map sync" => Some(
            "runtime_15_module_layout_status_docs_source_status_map_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output row-data module-layout child-summary guard child-owner split" => Some(
            "runtime_15_status_output_row_data_module_layout_child_summary_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout child-summary guard folder-backed split" => Some(
            "runtime_15_module_layout_child_summary_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout child-summary root inventory child split" => Some(
            "runtime_15_module_layout_child_summary_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout child-summary owner-budget guard child split" => Some(
            "runtime_15_module_layout_child_summary_owner_budget_guard_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout child-summary milestone-groups child split" => Some(
            "runtime_15_module_layout_child_summary_milestone_groups_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout child-summary foundation-review child split" => Some(
            "runtime_15_module_layout_child_summary_foundation_review_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout child-summary source/status-map sync" => Some(
            "runtime_15_module_layout_child_summary_source_status_map_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout child-summary status-doc guard child-owner split" => Some(
            "runtime_15_module_layout_child_summary_status_docs_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout child-summary status-doc guard folder-backed split" => Some(
            "runtime_15_module_layout_child_summary_status_docs_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout child-summary status-doc status-mirror child split" => Some(
            "runtime_15_module_layout_child_summary_status_docs_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout child-summary status-doc root inventory child split" => Some(
            "runtime_15_module_layout_child_summary_status_docs_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout child-summary status-doc source/status-map sync" => Some(
            "runtime_15_module_layout_child_summary_status_docs_source_status_map_sync_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

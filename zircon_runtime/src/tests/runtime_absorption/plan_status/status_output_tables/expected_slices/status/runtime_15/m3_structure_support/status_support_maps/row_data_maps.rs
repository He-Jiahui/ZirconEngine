pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status output M3 row data child-owner split" => {
            Some("runtime_15_status_output_m3_row_data_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 status output row-data guard child-owner split" => {
            Some("runtime_15_status_output_row_data_guard_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 status-output row-data module-layout guard folder-backed split" => Some(
            "runtime_15_status_output_row_data_module_layout_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout status-mirror child split" => Some(
            "runtime_15_module_layout_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout root inventory child split" => Some(
            "runtime_15_module_layout_root_inventory_child_split_static_passed_cargo_deferred",
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
        "Runtime 15 M3 status output review-guard row-data guard child-owner split" => Some(
            "runtime_15_status_output_review_guard_row_data_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data status-doc guard child-owner split" => Some(
            "runtime_15_review_guard_row_data_status_docs_child_owner_split_static_passed_cargo_deferred",
        ),
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
        "Runtime 15 M3 review-guard code-review row-data guard folder-backed split" => Some(
            "runtime_15_review_guard_code_review_rows_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard code-review row-data root inventory child split" => Some(
            "runtime_15_review_guard_code_review_rows_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard code-review status-mirror child split" => Some(
            "runtime_15_review_guard_code_review_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer status-output guard folder-backed split" => Some(
            "runtime_15_plugin_importer_status_output_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data guard folder-backed split" => Some(
            "runtime_15_review_guard_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data status-mirror child split" => Some(
            "runtime_15_review_guard_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data root inventory child split" => Some(
            "runtime_15_review_guard_row_data_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data status-doc guard folder-backed split" => Some(
            "runtime_15_review_guard_row_data_status_docs_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data status-doc status-mirror child split" => Some(
            "runtime_15_review_guard_row_data_status_docs_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data status-doc root inventory child split" => Some(
            "runtime_15_review_guard_row_data_status_docs_root_inventory_child_split_static_passed_cargo_deferred",
        ),
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
        "Runtime 15 M3 foundation-guards row-data guard folder-backed split" => Some(
            "runtime_15_foundation_guards_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation-guards row-data status-mirror child split" => Some(
            "runtime_15_foundation_guards_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation-guards row-data root inventory child split" => Some(
            "runtime_15_foundation_guards_row_data_root_inventory_child_split_static_passed_cargo_deferred",
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
        "Runtime 15 M3 child-groups status-doc guard child-owner split" => Some(
            "runtime_15_m3_child_groups_status_docs_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups status-doc guard folder-backed split" => Some(
            "runtime_15_m3_child_groups_status_docs_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups status-doc status-mirror child split" => Some(
            "runtime_15_m3_child_groups_status_docs_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups status-doc root inventory child split" => Some(
            "runtime_15_m3_child_groups_status_docs_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups row-data guard folder-backed split" => Some(
            "runtime_15_m3_child_groups_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups row-data status-mirror child split" => Some(
            "runtime_15_m3_child_groups_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups root inventory child split" => Some(
            "runtime_15_m3_child_groups_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups exports child split" => Some(
            "runtime_15_m3_child_groups_exports_child_split_static_passed_cargo_deferred",
        ),
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
        "Runtime 15 M3 lock-poison status row-data guard folder-backed split" => Some(
            "runtime_15_lock_poison_status_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 lock-poison status row-data status-mirror child split" => Some(
            "runtime_15_lock_poison_status_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 lock-poison status row-data root inventory child split" => Some(
            "runtime_15_lock_poison_status_row_data_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 scene-script row-data owner child split" => Some(
            "runtime_15_scene_script_row_data_owner_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 scene-script row-data guard folder-backed split" => Some(
            "runtime_15_scene_script_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 scene-script row-data status-mirror child split" => Some(
            "runtime_15_scene_script_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 scene-script row-data root inventory child split" => Some(
            "runtime_15_scene_script_row_data_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-group moved-row guard child-owner split" => Some(
            "runtime_15_m3_child_group_moved_row_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-group moved-row guard folder-backed split" => Some(
            "runtime_15_m3_child_group_moved_row_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-group moved-row status-mirror child split" => Some(
            "runtime_15_m3_child_group_moved_row_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-group moved-row root inventory child split" => Some(
            "runtime_15_m3_child_group_moved_row_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

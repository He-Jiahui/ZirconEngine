pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M2 row-data owner child split" {
        Some("2026-07-02")
        // Status: runtime_15_m2_row_data_owner_child_split_static_passed_cargo_deferred.
        // Files: plan_status/status_output_tables/expected_status_row_data/runtime_15/m2.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m2/core_scene_asset_dynamic.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m2/render_graphics.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m2/ui_platform_editor.rs.
        // Guard: runtime_15_m2_row_data_owner_is_child_backed.
    } else if slice == "Runtime 15 M3 M2 row-data children guard folder-backed split" {
        Some("2026-07-03")
        // Status: runtime_15_m2_row_data_children_guard_folder_backed_static_passed_cargo_deferred.
        // Files: structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children.rs; structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children/delegation.rs; structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children/row_ownership.rs; structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children/status_mirrors.rs; structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children/budgets.rs.
        // Guard: runtime_15_m2_row_data_children_guard_is_folder_backed.
    } else if slice == "Runtime 15 M3 M2 row-data children status-mirror child split" {
        Some("2026-07-04")
    } else if slice == "Runtime 15 M3 M2 row-data children root inventory child split" {
        Some("2026-07-04")
        // Status: runtime_15_m2_row_data_children_root_inventory_child_split_static_passed_cargo_deferred.
        // Files: structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children.rs; structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children/root_paths.rs; structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children/root_statuses.rs; structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children/root_child_rows.rs; structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children/root_owner_paths.rs; structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children/root_inventory.rs.
        // Guard: runtime_15_m2_row_data_children_root_inventory_is_child_owned.
    } else if slice == "Runtime 15 M1 animation manager folder-backed cutover" {
        // Status anchor:
        // runtime_15_animation_manager_folder_backed_cutover_static_passed_cargo_deferred.
        // Evidence anchors: animation/manager/mod.rs and animation/manager/graph.rs.
        // Guard anchor: runtime_15_animation_manager_is_folder_backed.
        Some("2026-06-24")
    } else if slice == "Runtime 15 M2 core runtime state module naming hard cutover" {
        // Status anchor:
        // runtime_15_core_runtime_state_module_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor: core/runtime/state/core_runtime_state.rs.
        // Guard anchor: runtime_15_core_runtime_state_module_uses_owner_name.
        Some("2026-06-24")
    } else {
        None
    }
}

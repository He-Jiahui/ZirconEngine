#[path = "status_support_maps/plan_doc_support_maps.rs"]
mod plan_doc_support_maps;
#[path = "status_support_maps/row_data_maps.rs"]
mod row_data_maps;

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(status) = row_data_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = plan_doc_support_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    match slice {
        "Runtime 15 M3 test file budget root-layout child split" => {
            Some("runtime_15_test_file_budget_root_layout_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 status output Runtime 15 row data split" => {
            Some("runtime_15_status_output_runtime_15_row_data_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 Runtime 15 row-data guard folder-backed split" => Some(
            "runtime_15_runtime_15_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 15 row-data status-mirror child split" => Some(
            "runtime_15_runtime_15_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 15 row-data row-ownership child split" => Some(
            "runtime_15_runtime_15_row_data_row_ownership_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 15 row-data root inventory child split" => Some(
            "runtime_15_runtime_15_row_data_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 foundation row data split" => Some(
            "runtime_15_status_output_runtime_15_foundation_row_data_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data topic child-owner split" => Some(
            "runtime_15_foundation_row_data_topic_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data 73-row docs sync" => Some(
            "runtime_15_foundation_row_data_71_row_docs_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data stale-count prose guard" => Some(
            "runtime_15_foundation_row_data_stale_count_prose_guard_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation row-data priority-doc frontmatter sync" => Some(
            "runtime_15_foundation_row_data_priority_doc_frontmatter_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 M2 row data split" => {
            Some("runtime_15_status_output_runtime_15_m2_row_data_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 M2 row-data guard child-owner split" => Some(
            "runtime_15_m2_row_data_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 M2 row-data guard folder-backed split" => Some(
            "runtime_15_m2_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 M2 row-data status-mirror child split" => Some(
            "runtime_15_m2_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 M2 row-data root inventory child split" => Some(
            "runtime_15_m2_row_data_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 support Hub project-actions tests child-owner split" => Some(
            "runtime_15_support_hub_project_actions_tests_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 support Hub runtime-state tests child-owner split" => Some(
            "runtime_15_support_hub_runtime_state_tests_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 support Hub view-model quick-actions/tests child-owner split" => Some(
            "runtime_15_support_hub_view_model_quick_actions_tests_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 editor retained-host workbench window projection tests child-owner split" => Some(
            "runtime_15_editor_retained_host_workbench_window_projection_tests_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 editor retained-host pane data conversion projection owner guard" => Some(
            "runtime_15_editor_retained_host_pane_data_conversion_owner_guard_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 production file budget core runtime guard split" => Some(
            "runtime_15_production_file_budget_core_runtime_guard_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 render shader template assembly guard support child-owner split" => Some(
            "runtime_15_render_shader_template_assembly_guard_support_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 render shader template assembly assertion contract child-owner split" => Some(
            "runtime_15_render_shader_template_assembly_assertion_contract_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 mesh pipeline shader source tests child-owner split" => Some(
            "runtime_15_mesh_pipeline_shader_source_tests_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 shader prewarm manifest guard child-owner split" => Some(
            "runtime_15_shader_prewarm_manifest_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 M4 row data split" => {
            Some("runtime_15_status_output_runtime_15_m4_row_data_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 Runtime 15 M4 row-data guard folder-backed split" => {
            Some("runtime_15_m4_row_data_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 Runtime 15 M4 row-data status-mirror child split" => Some(
            "runtime_15_m4_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 15 M4 row-data root inventory child split" => Some(
            "runtime_15_m4_row_data_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output expected-slice maps split" => {
            Some("runtime_15_status_output_expected_slice_maps_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner split" => Some(
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output expected-slice guard maps child-owner split" => Some(
            "runtime_15_status_output_expected_slice_guard_maps_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output expected-slice top-level map support child-owner split" => Some(
            "runtime_15_status_output_expected_slice_top_level_map_support_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output Runtime 15 M3 row data split" => {
            Some("runtime_15_status_output_runtime_15_m3_row_data_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 Runtime 15 M3 row-data guard folder-backed split" => {
            Some("runtime_15_m3_row_data_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 Runtime 15 M3 row-data status-mirror child split" => Some(
            "runtime_15_m3_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 15 M3 row-data root inventory child split" => Some(
            "runtime_15_m3_row_data_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 test file budget root-layout status scan child split" => Some(
            "runtime_15_test_file_budget_root_layout_status_scan_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 test file budget root-layout folder-backed guard child split" => Some(
            "runtime_15_test_file_budget_root_layout_folder_backed_guard_child_split_static_passed_cargo_timeout_no_result",
        ),
        "Runtime 15 M3 test file budget root-layout folder-backed support child-owner split" => Some(
            "runtime_15_test_file_budget_root_layout_folder_backed_support_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 root-layout status-output Runtime 15 row-data child-source sync" => Some(
            "runtime_15_root_layout_status_output_runtime_15_row_data_child_source_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 test file budget parent guard child-owner split" => Some(
            "runtime_15_test_file_budget_parent_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 historical oversized test roots closeout" => {
            Some("runtime_15_historical_oversized_test_roots_closeout_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 asset test-budget guard child-owner split" => {
            Some("runtime_15_asset_test_budget_guard_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 UI asset test folder split" => {
            Some("runtime_15_ui_asset_tests_folder_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 UI asset surface index test folder split" => {
            Some("runtime_15_ui_asset_surface_index_tests_folder_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 UI asset MUI web form style test folder split" => Some(
            "runtime_15_ui_asset_mui_web_form_style_tests_folder_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 UI asset MUI X web style test folder split" => Some(
            "runtime_15_ui_asset_mui_web_mui_x_style_tests_folder_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 UI asset MUI web style test folder split" => {
            Some("runtime_15_ui_asset_mui_web_style_tests_folder_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 UI taffy layout pass test folder split" => {
            Some("runtime_15_ui_taffy_layout_pass_tests_folder_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 UI runtime window input pump test folder split" => Some(
            "runtime_15_ui_runtime_window_input_pump_tests_folder_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 UI runtime window event ABI child folder split" => Some(
            "runtime_15_ui_runtime_window_event_abi_children_folder_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 test file budget root-layout UI child split" => {
            Some("runtime_15_test_file_budget_root_layout_ui_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 UI widget text input keyboard test folder split" => Some(
            "runtime_15_ui_widget_text_input_keyboard_tests_folder_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 UI focus navigation test folder split" => {
            Some("runtime_15_ui_focus_navigation_tests_folder_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 UI runtime input manager test folder split" => {
            Some("runtime_15_ui_runtime_input_manager_tests_folder_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 UI runtime input ownership test folder split" => Some(
            "runtime_15_ui_runtime_input_ownership_tests_folder_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 production file budget guard child-owner split" => Some(
            "runtime_15_production_file_budget_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output variable evidence anchors" => {
            Some("runtime_15_status_output_variable_evidence_anchors_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 status output evidence anchors guard folder-backed split" => Some(
            "runtime_15_status_output_evidence_anchors_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 evidence anchors status-mirror child split" => Some(
            "runtime_15_evidence_anchors_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 evidence anchors root inventory child split" => Some(
            "runtime_15_evidence_anchors_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

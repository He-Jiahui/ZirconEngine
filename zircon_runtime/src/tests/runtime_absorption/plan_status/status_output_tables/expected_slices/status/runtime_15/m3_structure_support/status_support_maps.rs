pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
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
        "Runtime 15 M3 status output row-data module-layout status-doc guard child-owner split" => Some(
            "runtime_15_status_output_row_data_module_layout_status_docs_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout status-doc guard folder-backed split" => Some(
            "runtime_15_module_layout_status_docs_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout status-doc status-mirror child split" => Some(
            "runtime_15_module_layout_status_docs_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output row-data module-layout child-summary guard child-owner split" => Some(
            "runtime_15_status_output_row_data_module_layout_child_summary_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 module-layout child-summary guard folder-backed split" => Some(
            "runtime_15_module_layout_child_summary_guard_folder_backed_static_passed_cargo_deferred",
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
        "Runtime 15 M3 review-guard code-review row-data guard folder-backed split" => Some(
            "runtime_15_review_guard_code_review_rows_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard code-review status-mirror child split" => Some(
            "runtime_15_review_guard_code_review_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data guard folder-backed split" => Some(
            "runtime_15_review_guard_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data status-doc guard folder-backed split" => Some(
            "runtime_15_review_guard_row_data_status_docs_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data status-doc status-mirror child split" => Some(
            "runtime_15_review_guard_row_data_status_docs_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard direct-assertion row-data guard folder-backed split" => Some(
            "runtime_15_review_guard_direct_assertion_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard direct-assertion status-mirror child split" => Some(
            "runtime_15_review_guard_direct_assertion_status_mirror_child_split_static_passed_cargo_deferred",
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
        "Runtime 15 M3 foundation-guards row-data guard folder-backed split" => Some(
            "runtime_15_foundation_guards_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 foundation-guards row-data status-mirror child split" => Some(
            "runtime_15_foundation_guards_row_data_status_mirror_child_split_static_passed_cargo_deferred",
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
        "Runtime 15 M3 child-groups status-doc guard child-owner split" => Some(
            "runtime_15_m3_child_groups_status_docs_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups status-doc guard folder-backed split" => Some(
            "runtime_15_m3_child_groups_status_docs_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups status-doc status-mirror child split" => Some(
            "runtime_15_m3_child_groups_status_docs_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups row-data guard folder-backed split" => Some(
            "runtime_15_m3_child_groups_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups row-data status-mirror child split" => Some(
            "runtime_15_m3_child_groups_row_data_status_mirror_child_split_static_passed_cargo_deferred",
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
        "Runtime 15 M3 lock-poison status row-data guard folder-backed split" => Some(
            "runtime_15_lock_poison_status_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 lock-poison status row-data status-mirror child split" => Some(
            "runtime_15_lock_poison_status_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 scene-script row-data guard folder-backed split" => Some(
            "runtime_15_scene_script_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 scene-script row-data status-mirror child split" => Some(
            "runtime_15_scene_script_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-group moved-row guard child-owner split" => Some(
            "runtime_15_m3_child_group_moved_row_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-group moved-row guard folder-backed split" => Some(
            "runtime_15_m3_child_group_moved_row_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output expected-slice legacy child-owner split" => Some(
            "runtime_15_status_output_expected_slice_legacy_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output expected-slice legacy group child-owner split" => Some(
            "runtime_15_status_output_expected_slice_legacy_group_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output expected-slice guard child-owner split" => Some(
            "runtime_15_status_output_expected_slice_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 structure-support expected-slice map child-owner split" => Some(
            "runtime_15_structure_support_expected_slice_map_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 structure-convention warning cleanup" => Some(
            "runtime_15_structure_convention_warning_cleanup_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 runtime index subplan map 01-15 sync" => Some(
            "runtime_15_runtime_index_subplan_map_01_15_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 runtime index problem-row parser P01-P17 sync" => Some(
            "runtime_15_runtime_index_problem_row_parser_p01_p17_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 03 module-doc status index anchor sync" => Some(
            "runtime_15_runtime_03_module_doc_status_index_anchor_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 scene asset status anchor sync" => Some(
            "runtime_15_runtime_07_scene_asset_status_anchor_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 07 owner-budget status anchor sync" => Some(
            "runtime_15_runtime_07_owner_budget_status_anchor_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 02 generated status anchor sync" => Some(
            "runtime_15_runtime_02_generated_status_anchor_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 10 behavior status anchor sync" => Some(
            "runtime_15_runtime_10_behavior_status_anchor_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime Cargo attempt status anchor sync" => Some(
            "runtime_15_runtime_cargo_attempt_status_anchor_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plan-status index-tables child-owner split" => Some(
            "runtime_15_plan_status_index_tables_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plan-status support inventory review sync" => Some(
            "runtime_15_plan_status_support_inventory_review_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs code-path integrity guard" => Some(
            "runtime_15_priority_plan_docs_code_path_integrity_guard_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs test-path integrity guard" => Some(
            "runtime_15_priority_plan_docs_test_path_integrity_guard_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs frontmatter status guard" => Some(
            "runtime_15_priority_plan_docs_frontmatter_status_guard_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs frontmatter uniqueness guard" => Some(
            "runtime_15_priority_plan_docs_frontmatter_uniqueness_guard_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs required header sections guard" => Some(
            "runtime_15_priority_plan_docs_required_header_sections_guard_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs plan-source cross-link guard" => Some(
            "runtime_15_priority_plan_docs_plan_source_cross_link_guard_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs guard-test listing guard" => Some(
            "runtime_15_priority_plan_docs_guard_test_listing_guard_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs guard child-owner split" => Some(
            "runtime_15_priority_plan_docs_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs child prose full inventory sync" => Some(
            "runtime_15_priority_plan_docs_child_prose_full_inventory_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs guard-test child-owner split" => Some(
            "runtime_15_priority_plan_docs_guard_test_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs guard-test child prose full inventory sync" => Some(
            "runtime_15_priority_plan_docs_guard_test_child_prose_full_inventory_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs moved guard path mirror" => Some(
            "runtime_15_priority_plan_docs_moved_guard_path_mirror_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs guard inventory row-data source sync" => Some(
            "runtime_15_priority_plan_docs_guard_inventory_row_data_source_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs listing prose full inventory sync" => Some(
            "runtime_15_priority_plan_docs_listing_prose_full_inventory_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs moved mirror full inventory sync" => Some(
            "runtime_15_priority_plan_docs_moved_mirror_full_inventory_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs row-data owner child split" => Some(
            "runtime_15_priority_plan_docs_row_data_owner_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs owner-guard row-data child split" => Some(
            "runtime_15_priority_plan_docs_owner_guard_row_data_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs row-data guard folder-backed split" => Some(
            "runtime_15_priority_plan_docs_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs status-mirror child split" => Some(
            "runtime_15_priority_plan_docs_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output owner stale-path follow-up" => Some(
            "runtime_15_status_output_owner_stale_path_followup_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support row-data owner child split" => Some(
            "runtime_15_status_support_row_data_owner_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support row-data guard folder-backed split" => Some(
            "runtime_15_status_support_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        // Files: plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs.rs.
        // Guard: runtime_15_status_support_row_data_owner_is_child_backed.
        "Runtime 15 M3 asset-budget row-data owner child split" => Some(
            "runtime_15_asset_budget_row_data_owner_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 asset-budget row-data guard folder-backed split" => Some(
            "runtime_15_asset_budget_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        // Files: plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests/runtime_rhi.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests/asset_tests.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests/budget_render_ui.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests/naming_core_asset_dynamic.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests/naming_graphics_misc.rs.
        // Guard: runtime_15_asset_budget_row_data_owner_is_child_backed.
        // Files: structure_convention/production_file_budget/render_shader_template_assembly.rs; structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions.rs; structure_convention/production_file_budget/render_shader_template_assembly/docs_anchors.rs; structure_convention/production_file_budget/render_shader_template_assembly/sources.rs.
        // Guard: runtime_15_render_shader_template_assembly_support_children_are_folder_backed.
        // Files: structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions.rs; structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/template_contracts.rs; structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/mesh_cache_contracts.rs; structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/mesh_pipeline_shadow_graph_contracts.rs; structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/owner_budget.rs.
        // Guard: runtime_15_render_shader_template_assembly_support_children_are_folder_backed.
        // Files: graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs; graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs; graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests/runtime_shading_model_sources.rs.
        // Guard: runtime_15_render_shader_template_assembly_support_children_are_folder_backed.
        // Files: plan_status/index_tables.rs; plan_status/index_tables/subplan_map.rs; plan_status/index_tables/status_anchors.rs; plan_status/index_tables/index_consistency.rs.
        // Guard: runtime_15_plan_status_index_tables_guard_child_owner_split.
        // Guard: runtime_architecture_review_documents_all_absorption_guards.
        // Guard: runtime_15_runtime_cargo_attempt_status_index_anchors_are_locked.
        // Guard: runtime_15_runtime_10_behavior_status_index_anchors_are_locked.
        // Guard: runtime_15_runtime_02_generated_status_index_anchors_are_locked.
        // Guard: runtime_15_runtime_07_owner_budget_status_index_anchors_are_locked.
        // Guard: runtime_15_runtime_07_scene_asset_status_index_anchors_are_locked.
        // Guard: runtime_15_runtime_03_module_doc_status_index_anchors_are_locked.
        // Guard: runtime_15_runtime_index_problem_row_parser_covers_p01_p17_status_locked.
        // Guard: runtime_15_runtime_index_subplan_map_covers_01_15_status_locked.
        _ => None,
    }
}

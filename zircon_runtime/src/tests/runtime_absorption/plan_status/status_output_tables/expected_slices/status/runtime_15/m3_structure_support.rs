pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M3 graphics dead-code guard module split" {
        Some("runtime_15_graphics_dead_code_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 graphics dead-code guard child-owner split" {
        Some("runtime_15_graphics_dead_code_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 graphics dead-code guard forbidden attribute literal cleanup"
    {
        Some("runtime_15_graphics_dead_code_guard_literal_cleanup_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 provider boilerplate guard module split" {
        Some("runtime_15_provider_boilerplate_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 provider boilerplate guard child-owner split" {
        Some("runtime_15_provider_boilerplate_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 facade surface guard module split" {
        Some("runtime_15_facade_surface_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 runtime dead-code guard module split" {
        Some("runtime_15_runtime_dead_code_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 runtime dead-code guard forbidden attribute literal cleanup" {
        Some("runtime_15_runtime_dead_code_guard_literal_cleanup_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 diagnostics guard module split" {
        Some("runtime_15_diagnostics_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 core framework test folder split" {
        Some("runtime_15_core_framework_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 picking test folder split" {
        Some("runtime_15_picking_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 core runtime deactivation blocked test folder split" {
        Some(
            "runtime_15_core_runtime_deactivation_blocked_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 lock-poison status row-data child-owner split" {
        Some(
            "runtime_15_lock_poison_status_row_data_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 module-convention status row-data child-owner split" {
        Some(
            "runtime_15_module_convention_status_row_data_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 module convention gate output contract" {
        Some("runtime_15_module_convention_gate_output_contract_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 module convention non-render debt guard" {
        Some("runtime_15_module_convention_non_render_debt_guard_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 render-scoped migration debt handoff gate" {
        Some("runtime_15_render_scoped_migration_debt_handoff_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 hard-cutover allowed Hyper policy risk cleanup" {
        Some(
            "runtime_15_hard_cutover_allowed_hyper_policy_risk_cleanup_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 module convention gate audit-clear status mirror" {
        Some(
            "runtime_15_module_convention_gate_audit_clear_status_mirror_core_min_cargo_passed_full_sweep_pending",
        )
    } else if slice == "Runtime 15 M3 module convention audit script family naming cleanup" {
        Some(
            "runtime_15_module_convention_audit_script_family_naming_core_min_cargo_passed_full_sweep_pending",
        )
    } else if slice == "Runtime 15 M3 code review findings test folder split" {
        Some("runtime_15_code_review_findings_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 review top-row status row-data child-owner split" {
        Some(
            "runtime_15_review_top_row_status_row_data_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 D-S7 static plugin manifest generation/parity review sync" {
        Some(
            "ds7_static_plugin_manifest_generation_parity_review_synced_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 D7 core workspace dependency top-row closed status sync" {
        Some("d7_core_workspace_dependency_top_row_closed_status_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 D7 core workspace dependency inheritance guard" {
        Some("d7_core_workspace_dependency_inheritance_guard_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 D8 runtime registration builder original evidence paths" {
        Some("d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 D6 RuntimePluginId open string-newtype review sync" {
        Some("d6_runtime_plugin_id_open_string_newtype_review_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 F5/F6/F7 typed-error top-row closed status sync" {
        Some("f5_f6_f7_typed_error_top_row_closed_status_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 F8/F9/F10 runtime surface top-row closed status sync" {
        Some("f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 F13/F14 provider diagnostics top-row closed status sync" {
        Some("f13_f14_provider_diagnostics_top_row_closed_status_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 F17/F18 lookup/manager top-row closed status sync" {
        Some("f17_f18_lookup_manager_top_row_closed_status_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 F19 scene renderer construction top-row closed status sync" {
        Some("f19_scene_renderer_construction_top_row_closed_status_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 D9 editor/runtime mirror consumer guard" {
        Some("d9_editor_runtime_mirror_consumers_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 D5 editor authoring macro consumer guard" {
        Some("d5_editor_authoring_macro_consumers_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 typed-error convergence guard child-owner split" {
        Some(
            "runtime_15_typed_error_convergence_guard_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 dynamic scene absorption guard folder split" {
        Some("runtime_15_dynamic_scene_absorption_guard_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 input manager test folder split" {
        Some("runtime_15_input_manager_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI architecture test folder split" {
        Some("runtime_15_ui_architecture_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI v2 asset test folder split" {
        Some("runtime_15_ui_v2_asset_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 UI shared core test folder split" {
        Some("runtime_15_ui_shared_core_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 UI shared core guard child-owner split" {
        Some("runtime_15_ui_shared_core_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI shared core input visibility child folder split" {
        Some(
            "runtime_15_ui_shared_core_input_visibility_child_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI shared core scroll mutation child folder split" {
        Some(
            "runtime_15_ui_shared_core_scroll_mutation_child_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI shared core layout surface child folder split" {
        Some(
            "runtime_15_ui_shared_core_layout_surface_child_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI accessibility test folder split" {
        Some("runtime_15_ui_accessibility_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI accessibility widget actions test folder split" {
        Some(
            "runtime_15_ui_accessibility_widget_actions_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI layout slots test folder split" {
        Some("runtime_15_ui_layout_slots_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI surface-frame authority test folder split" {
        Some(
            "runtime_15_ui_surface_frame_authority_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI surface dirty domains test folder split" {
        Some("runtime_15_ui_surface_dirty_domains_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI material layout test folder split" {
        Some("runtime_15_ui_material_layout_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI template test folder split" {
        Some("runtime_15_ui_template_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI component catalog test folder split" {
        Some("runtime_15_ui_component_catalog_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI boundary test folder split" {
        Some("runtime_15_ui_boundary_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI component state test folder split" {
        Some(
            "runtime_15_ui_component_catalog_component_state_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI component state keyboard test folder split" {
        Some(
            "runtime_15_ui_component_catalog_component_state_keyboard_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI Material foundation test folder split" {
        Some(
            "runtime_15_ui_component_catalog_material_foundation_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI event routing test folder split" {
        Some("runtime_15_ui_event_routing_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI runtime input reply routes test folder split" {
        Some(
            "runtime_15_ui_runtime_input_reply_routes_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI runtime input reply route child folder split" {
        Some(
            "runtime_15_ui_runtime_input_reply_route_children_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI runtime input reply table pointer route folder split" {
        Some(
            "runtime_15_ui_runtime_input_reply_table_pointer_routes_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI runtime input reply route guard child-owner split" {
        Some(
            "runtime_15_ui_runtime_input_reply_route_guard_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 runtime diagnostics test folder split" {
        Some("runtime_15_runtime_diagnostics_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 RHI command list test folder split" {
        Some("runtime_15_rhi_command_list_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 RHI device contract test folder split" {
        Some("runtime_15_rhi_device_contract_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset pack test folder split" {
        Some("runtime_15_asset_pack_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset facade test folder split" {
        Some("runtime_15_asset_facade_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset project zmeta test folder split" {
        Some("runtime_15_asset_project_zmeta_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset project manager test folder split" {
        Some("runtime_15_asset_project_manager_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset project flow sample test folder split" {
        Some(
            "runtime_15_asset_project_flow_sample_tests_folder_split_static_passed_cargo_lock_blocked",
        )
    } else if slice == "Runtime 15 M3 asset project example vampire test folder split" {
        Some(
            "runtime_15_asset_project_example_vampire_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 asset artifact store test folder split" {
        Some("runtime_15_asset_artifact_store_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 asset material test folder split" {
        Some("runtime_15_asset_material_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset mesh test root split" {
        Some("runtime_15_asset_mesh_tests_root_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 asset glTF importer test folder split" {
        Some("runtime_15_asset_gltf_importer_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset glTF primitive fixture folder split" {
        Some(
            "runtime_15_asset_gltf_primitive_fixtures_folder_split_static_passed_cargo_lock_blocked",
        )
    } else if slice == "Runtime 15 M3 asset importer test folder split" {
        Some("runtime_15_asset_importer_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset scene test folder split" {
        Some("runtime_15_asset_scene_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset UI test folder split" {
        Some("runtime_15_asset_ui_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 asset pipeline manager test folder split" {
        Some("runtime_15_asset_pipeline_manager_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 test file budget guard folder split" {
        Some("runtime_15_test_file_budget_guard_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 test file budget guard root mod cutover" {
        Some("runtime_15_test_file_budget_guard_root_mod_cutover_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 no oversized test files global gate" {
        Some("runtime_15_no_oversized_test_files_global_gate_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 Runtime 07 performance hotspot guard folder split" {
        Some(
            "runtime_15_runtime_07_performance_hotspots_guard_folder_split_static_passed_cargo_timeout_no_result",
        )
    } else if slice == "Runtime 15 M3 script VM test folder split" {
        Some("runtime_15_script_vm_tests_folder_split_static_passed_cargo_timeout_no_result")
    } else if slice == "Runtime 15 M3 script VM hot-reload coordinator test folder split" {
        Some(
            "runtime_15_script_vm_hot_reload_coordinator_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 native live-host tests folder split" {
        Some("runtime_15_native_live_host_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 native plugin loader real fixture test folder split" {
        Some(
            "runtime_15_native_plugin_loader_real_fixture_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 extension registry bridge test folder split" {
        Some("runtime_15_extension_registry_bridge_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 manifest contributions test folder split" {
        Some("runtime_15_manifest_contributions_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 runtime plugin package manifest test folder split" {
        Some(
            "runtime_15_runtime_plugin_package_manifest_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 export build plan test folder split" {
        Some("runtime_15_export_build_plan_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 export build plan platform test folder split" {
        Some(
            "runtime_15_export_build_plan_platform_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 gameplay host test folder split" {
        Some("runtime_15_gameplay_host_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 shader prewarm manifest test folder split" {
        Some("runtime_15_shader_prewarm_manifest_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS schedule test folder split" {
        Some("runtime_15_scene_ecs_schedule_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS schedule conflict graph child folder split" {
        Some(
            "runtime_15_scene_ecs_schedule_conflict_graph_child_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 scene ECS systems test folder split" {
        Some("runtime_15_scene_ecs_systems_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS query test folder split" {
        Some("runtime_15_scene_ecs_query_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS query structure test folder split" {
        Some("runtime_15_scene_ecs_query_structure_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene derived-state test folder split" {
        Some("runtime_15_scene_derived_state_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 dynamic scene session path-management test folder split" {
        Some(
            "runtime_15_dynamic_scene_session_path_management_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 scene component-structure test folder split" {
        Some("runtime_15_scene_component_structure_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS reflect foundation test folder split" {
        Some(
            "runtime_15_scene_ecs_reflect_foundation_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 dynamic scene root test folder split" {
        Some("runtime_15_dynamic_scene_root_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene render extract test folder split" {
        Some("runtime_15_scene_render_extract_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene asset integration test folder split" {
        Some("runtime_15_scene_asset_integration_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene world basics test folder split" {
        Some("runtime_15_scene_world_basics_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene property paths test folder split" {
        Some("runtime_15_scene_property_paths_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 test file budget root-layout child split" {
        Some("runtime_15_test_file_budget_root_layout_child_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 status output Runtime 15 row data split" {
        Some("runtime_15_status_output_runtime_15_row_data_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 status output Runtime 15 foundation row data split" {
        Some(
            "runtime_15_status_output_runtime_15_foundation_row_data_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 status output Runtime 15 M2 row data split" {
        Some("runtime_15_status_output_runtime_15_m2_row_data_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 support Hub project-actions tests child-owner split" {
        Some(
            "runtime_15_support_hub_project_actions_tests_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 support Hub runtime-state tests child-owner split" {
        Some(
            "runtime_15_support_hub_runtime_state_tests_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 support Hub view-model quick-actions/tests child-owner split"
    {
        Some(
            "runtime_15_support_hub_view_model_quick_actions_tests_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice
        == "Runtime 15 M3 editor retained-host workbench window projection tests child-owner split"
    {
        Some(
            "runtime_15_editor_retained_host_workbench_window_projection_tests_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice
        == "Runtime 15 M3 editor retained-host pane data conversion projection owner guard"
    {
        Some(
            "runtime_15_editor_retained_host_pane_data_conversion_owner_guard_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 production file budget core runtime guard split" {
        Some(
            "runtime_15_production_file_budget_core_runtime_guard_split_static_passed_cargo_deferred",
        )
    } else if slice
        == "Runtime 15 M3 render shader template assembly guard support child-owner split"
    {
        Some(
            "runtime_15_render_shader_template_assembly_guard_support_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 shader prewarm manifest guard child-owner split" {
        Some(
            "runtime_15_shader_prewarm_manifest_guard_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 status output Runtime 15 M4 row data split" {
        Some("runtime_15_status_output_runtime_15_m4_row_data_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 status output expected-slice maps split" {
        Some("runtime_15_status_output_expected_slice_maps_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 status output Runtime 15 expected-slice child-owner split" {
        Some(
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 status output expected-slice guard maps child-owner split" {
        Some(
            "runtime_15_status_output_expected_slice_guard_maps_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice
        == "Runtime 15 M3 status output expected-slice top-level map support child-owner split"
    {
        Some(
            "runtime_15_status_output_expected_slice_top_level_map_support_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 status output Runtime 15 M3 row data split" {
        Some("runtime_15_status_output_runtime_15_m3_row_data_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 test file budget root-layout status scan child split" {
        Some(
            "runtime_15_test_file_budget_root_layout_status_scan_child_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 test file budget root-layout folder-backed guard child split"
    {
        Some(
            "runtime_15_test_file_budget_root_layout_folder_backed_guard_child_split_static_passed_cargo_timeout_no_result",
        )
    } else if slice
        == "Runtime 15 M3 test file budget root-layout folder-backed support child-owner split"
    {
        Some(
            "runtime_15_test_file_budget_root_layout_folder_backed_support_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 test file budget parent guard child-owner split" {
        Some(
            "runtime_15_test_file_budget_parent_guard_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 historical oversized test roots closeout" {
        Some("runtime_15_historical_oversized_test_roots_closeout_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 asset test-budget guard child-owner split" {
        Some("runtime_15_asset_test_budget_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI asset test folder split" {
        Some("runtime_15_ui_asset_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI asset surface index test folder split" {
        Some("runtime_15_ui_asset_surface_index_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI asset MUI web form style test folder split" {
        Some(
            "runtime_15_ui_asset_mui_web_form_style_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI asset MUI X web style test folder split" {
        Some(
            "runtime_15_ui_asset_mui_web_mui_x_style_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI asset MUI web style test folder split" {
        Some("runtime_15_ui_asset_mui_web_style_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI taffy layout pass test folder split" {
        Some("runtime_15_ui_taffy_layout_pass_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI runtime window input pump test folder split" {
        Some(
            "runtime_15_ui_runtime_window_input_pump_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI runtime window event ABI child folder split" {
        Some(
            "runtime_15_ui_runtime_window_event_abi_children_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 test file budget root-layout UI child split" {
        Some("runtime_15_test_file_budget_root_layout_ui_child_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI widget text input keyboard test folder split" {
        Some(
            "runtime_15_ui_widget_text_input_keyboard_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI focus navigation test folder split" {
        Some("runtime_15_ui_focus_navigation_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI runtime input manager test folder split" {
        Some("runtime_15_ui_runtime_input_manager_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI runtime input ownership test folder split" {
        Some(
            "runtime_15_ui_runtime_input_ownership_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 production file budget guard child-owner split" {
        Some(
            "runtime_15_production_file_budget_guard_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 status output variable evidence anchors" {
        Some("runtime_15_status_output_variable_evidence_anchors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 status output M3 row data child-owner split" {
        Some("runtime_15_status_output_m3_row_data_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 status output row-data guard child-owner split" {
        Some(
            "runtime_15_status_output_row_data_guard_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 status output expected-slice legacy child-owner split" {
        Some(
            "runtime_15_status_output_expected_slice_legacy_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 status output expected-slice legacy group child-owner split" {
        Some(
            "runtime_15_status_output_expected_slice_legacy_group_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 status output expected-slice guard child-owner split" {
        Some(
            "runtime_15_status_output_expected_slice_guard_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 D12 runtime helper export macro review sync" {
        Some("d12_runtime_export_macro_review_synced_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 D1 capability single-source review sync" {
        Some("d1_capability_single_source_review_synced_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 D10 animation/physics bridge call migration" {
        Some("d10_animation_physics_bridge_call_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 D11 animation/physics TestRuntime fixture migration" {
        Some("d11_animation_physics_test_runtime_fixture_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 D13 importer manifest parity guard" {
        Some("d13_importer_manifest_parity_guard_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 P0/DX priority D13 parity sync" {
        Some("review_priority_recommendation_d13_parity_sync_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 D13 importer top-row closed status sync" {
        Some("d13_importer_top_row_closed_status_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 D-S8/D3 native fixture top-row closed status sync" {
        Some("ds8_d3_native_fixture_top_row_closed_status_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 P0 F1/F2/F4 top-row closed status sync" {
        Some("p0_f1_f2_f4_top_row_closed_status_static_passed_cargo_deferred")
    } else {
        None
    }
}

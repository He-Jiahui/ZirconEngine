pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 F9 runtime prelude required type coverage" {
        Some("runtime_15_prelude_required_types_coremin_check_passed")
    } else if slice == "Runtime 15 runtime UI dead-code support split" {
        Some("runtime_15_runtime_ui_dead_code_support_split_coremin_check_passed")
    } else if slice == "Runtime 15 graphics facade visibility note" {
        Some(
            "runtime_15_graphics_facade_visibility_note_static_passed_cargo_blocked_graphics_drift",
        )
    } else if slice == "Runtime 15 F14 diagnostics normalization" {
        Some("runtime_15_diagnostics_frame_trait_wrapper_removed_coremin_check_passed")
    } else if slice == "Runtime 15 F13 provider registration shared owner" {
        Some("runtime_15_provider_registration_shared_owner_coremin_check_passed")
    } else if slice == "Runtime 15 F13 provider update shared stats owner" {
        Some("runtime_15_provider_update_shared_stats_owner_coremin_check_passed")
    } else if slice == "Runtime 15 F13 provider feedback shared payload owner" {
        Some("runtime_15_provider_feedback_shared_payload_owner_coremin_check_passed")
    } else if slice == "Runtime 15 F13 provider prepare input shared frame owner" {
        Some("runtime_15_provider_prepare_input_shared_frame_owner_coremin_check_passed")
    } else if slice == "Runtime 15 F13 full provider boilerplate audit" {
        Some("runtime_15_provider_boilerplate_full_audit_coremin_check_passed")
    } else if slice == "Runtime 15 F12 runtime-owned dead-code suppression cleanup" {
        Some("runtime_15_runtime_owned_dead_code_suppression_cleanup_coremin_check_passed")
    } else if slice == "Runtime 15 F12 script host value descriptor dead-code cleanup" {
        Some("runtime_15_script_host_value_descriptors_coremin_check_passed")
    } else if slice == "Runtime 15 F12 script reflection macro fixture dead-code cleanup" {
        Some("runtime_15_script_reflection_macro_fixture_dead_code_cleanup_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 core runtime service-list owner split" {
        Some("runtime_15_core_runtime_service_lists_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M4 RHI WGPU command validation render-state owner split" {
        Some("runtime_15_rhi_wgpu_command_validation_render_state_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M4 RHI WGPU UI surface render/setup owner split" {
        Some("runtime_15_rhi_wgpu_ui_surface_render_setup_owner_split_static_passed_cargo_timeout_no_result")
    } else if slice == "Runtime 15 M4 RHI WGPU UI surface geometry test owner split" {
        Some(
            "runtime_15_rhi_wgpu_ui_surface_geometry_tests_owner_split_static_passed_cargo_timeout_no_result",
        )
    } else if slice == "Runtime 15 M4 material asset value/readiness helper owner split" {
        Some("runtime_15_material_asset_value_readiness_owner_split_static_passed_cargo_timeout_no_result")
    } else if slice
        == "Runtime 15 M4 core runtime render-stats graph execution-resources owner split"
    {
        Some("runtime_15_render_stats_graph_execution_resources_owner_split_static_passed_cargo_timeout_no_result")
    } else if slice == "Runtime 15 M4 scene fixed light reflection write-field owner split" {
        Some("runtime_15_scene_fixed_light_reflection_write_fields_owner_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M4 scene world property-access physics write owner split" {
        Some("runtime_15_scene_world_property_access_physics_owner_split_static_passed_cargo_timeout_no_result")
    } else if slice == "Runtime 15 M4 scene world property-access physics entry owner split" {
        Some("runtime_15_scene_world_property_access_physics_entries_owner_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M4 scene world project I/O mesh owner split" {
        Some("runtime_15_scene_world_project_io_mesh_owner_split_static_passed_cargo_timeout_no_result")
    } else if slice == "Runtime 15 M4 UI text layout engine visual-order owner split" {
        Some(
            "runtime_15_ui_text_layout_engine_visual_order_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M4 UI layout arrange grid/masonry owner split" {
        Some("runtime_15_ui_layout_arrange_grid_masonry_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 UI template MUI X DataGrid class owner split" {
        Some(
            "runtime_15_ui_template_mui_x_data_grid_class_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M4 UI template document validation owner split" {
        Some("runtime_15_ui_template_document_validation_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 UI template style slot-contract owner split" {
        Some("runtime_15_ui_template_style_slot_contract_owner_split_static_passed_cargo_timeout_no_result")
    } else if slice == "Runtime 15 M4 UI v2 style runtime-state owner split" {
        Some("runtime_15_ui_v2_style_runtime_state_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 UI accessibility extract state owner split" {
        Some("runtime_15_ui_accessibility_extract_state_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 UI component catalog editor-showcase helper owner split" {
        Some(
            "runtime_15_ui_component_catalog_editor_showcase_helper_owner_split_static_passed_cargo_timeout_no_result",
        )
    } else if slice == "Runtime 15 M4 UI component state-reducer keyboard menu submenu owner split"
    {
        Some("runtime_15_ui_component_state_reducer_keyboard_menu_submenu_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 UI component state-reducer tree view editing owner split" {
        Some("runtime_15_ui_component_state_reducer_tree_view_editing_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 UI surface event-routing owner split" {
        Some("runtime_15_ui_surface_event_routing_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 UI surface property mutation metadata dirty owner split" {
        Some("runtime_15_ui_surface_property_mutation_metadata_dirty_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 UI surface render feedback command/color owner split" {
        Some(
            "runtime_15_ui_surface_render_feedback_command_color_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M4 UI surface default-interactions keyboard/timer owner split" {
        Some("runtime_15_ui_surface_default_interactions_keyboard_timer_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 UI surface table column helper owner split" {
        Some("runtime_15_ui_surface_table_column_helper_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F12 offscreen target texture owner cleanup" {
        Some("runtime_15_offscreen_target_texture_owner_cleanup_static_passed_cargo_timeout_no_result")
    } else if slice == "Runtime 15 F12 render backend state owner cleanup" {
        Some("runtime_15_render_backend_state_owner_cleanup_coremin_check_passed")
    } else if slice == "Runtime 15 F12 gpu texture resource owner cleanup" {
        Some("runtime_15_gpu_texture_resource_owner_cleanup_coremin_check_passed")
    } else if slice == "Runtime 15 F12 gpu material uniform owner cleanup" {
        Some("runtime_15_gpu_material_uniform_owner_cleanup_coremin_check_passed")
    } else if slice == "Runtime 15 F12 gpu mesh order signature cleanup" {
        Some("runtime_15_gpu_mesh_order_signature_cleanup_coremin_check_passed")
    } else if slice == "Runtime 15 F12 gpu model identity cleanup" {
        Some("runtime_15_gpu_model_identity_cleanup_coremin_check_passed")
    } else if slice == "Runtime 15 F12 post-process LUT texture owner cleanup" {
        Some("runtime_15_post_process_lut_texture_owner_cleanup_coremin_check_passed")
    } else if slice == "Runtime 15 F12 output target texture owner cleanup" {
        Some("runtime_15_output_target_texture_owner_cleanup_coremin_check_passed")
    } else if slice == "Runtime 15 F12 material runtime capture seed cleanup" {
        Some("runtime_15_material_runtime_capture_seed_cleanup_coremin_check_passed")
    } else if slice == "Runtime 15 F12 resource streamer diagnostics accessor cleanup" {
        Some("runtime_15_resource_streamer_diagnostics_accessor_cleanup_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 F12 resource streamer resolve texture id cleanup" {
        Some("runtime_15_resource_streamer_resolve_texture_id_cleanup_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 F12 particle GPU readback output accessor cleanup" {
        Some("runtime_15_particle_gpu_readback_output_accessor_cleanup_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 F12 advanced plugin output test accessor cleanup" {
        Some("runtime_15_advanced_plugin_output_test_accessor_cleanup_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 graphics dead-code guard module split" {
        Some("runtime_15_graphics_dead_code_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 graphics dead-code guard child-owner split" {
        Some("runtime_15_graphics_dead_code_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 provider boilerplate guard module split" {
        Some("runtime_15_provider_boilerplate_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 facade surface guard module split" {
        Some("runtime_15_facade_surface_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 runtime dead-code guard module split" {
        Some("runtime_15_runtime_dead_code_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 diagnostics guard module split" {
        Some("runtime_15_diagnostics_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 core framework test folder split" {
        Some("runtime_15_core_framework_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 core runtime deactivation blocked test folder split" {
        Some("runtime_15_core_runtime_deactivation_blocked_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 code review findings test folder split" {
        Some("runtime_15_code_review_findings_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 dynamic scene absorption guard folder split" {
        Some("runtime_15_dynamic_scene_absorption_guard_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI architecture test folder split" {
        Some("runtime_15_ui_architecture_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI v2 asset test folder split" {
        Some("runtime_15_ui_v2_asset_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 UI shared core test folder split" {
        Some("runtime_15_ui_shared_core_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 UI accessibility test folder split" {
        Some("runtime_15_ui_accessibility_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI accessibility widget actions test folder split" {
        Some("runtime_15_ui_accessibility_widget_actions_tests_folder_split_static_passed_cargo_deferred")
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
        Some("runtime_15_ui_component_catalog_component_state_keyboard_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI Material foundation test folder split" {
        Some(
            "runtime_15_ui_component_catalog_material_foundation_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 UI event routing test folder split" {
        Some("runtime_15_ui_event_routing_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI runtime input reply routes test folder split" {
        Some("runtime_15_ui_runtime_input_reply_routes_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI runtime input reply route child folder split" {
        Some("runtime_15_ui_runtime_input_reply_route_children_folder_split_static_passed_cargo_deferred")
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
        Some("runtime_15_asset_project_flow_sample_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset material test folder split" {
        Some("runtime_15_asset_material_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset glTF importer test folder split" {
        Some("runtime_15_asset_gltf_importer_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset glTF primitive fixture folder split" {
        Some("runtime_15_asset_gltf_primitive_fixtures_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset importer test folder split" {
        Some("runtime_15_asset_importer_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset scene test folder split" {
        Some("runtime_15_asset_scene_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 test file budget guard folder split" {
        Some("runtime_15_test_file_budget_guard_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 test file budget guard root mod cutover" {
        Some("runtime_15_test_file_budget_guard_root_mod_cutover_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 Runtime 07 performance hotspot guard folder split" {
        Some("runtime_15_runtime_07_performance_hotspots_guard_folder_split_static_passed_cargo_timeout_no_result")
    } else if slice == "Runtime 15 M3 script VM test folder split" {
        Some("runtime_15_script_vm_tests_folder_split_static_passed_cargo_timeout_no_result")
    } else if slice == "Runtime 15 M3 gameplay host test folder split" {
        Some("runtime_15_gameplay_host_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 shader prewarm manifest test folder split" {
        Some("runtime_15_shader_prewarm_manifest_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS schedule test folder split" {
        Some("runtime_15_scene_ecs_schedule_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS systems test folder split" {
        Some("runtime_15_scene_ecs_systems_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS query test folder split" {
        Some("runtime_15_scene_ecs_query_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS query structure test folder split" {
        Some("runtime_15_scene_ecs_query_structure_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene derived-state test folder split" {
        Some("runtime_15_scene_derived_state_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 dynamic scene session path-management test folder split" {
        Some("runtime_15_dynamic_scene_session_path_management_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene component-structure test folder split" {
        Some("runtime_15_scene_component_structure_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 scene ECS reflect foundation test folder split" {
        Some("runtime_15_scene_ecs_reflect_foundation_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 dynamic scene root test folder split" {
        Some("runtime_15_dynamic_scene_root_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 test file budget root-layout child split" {
        Some("runtime_15_test_file_budget_root_layout_child_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 status output Runtime 15 row data split" {
        Some("runtime_15_status_output_runtime_15_row_data_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 production file budget core runtime guard split" {
        Some("runtime_15_production_file_budget_core_runtime_guard_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 status output Runtime 15 M4 row data split" {
        Some("runtime_15_status_output_runtime_15_m4_row_data_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 status output expected-slice maps split" {
        Some("runtime_15_status_output_expected_slice_maps_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 status output Runtime 15 M3 row data split" {
        Some("runtime_15_status_output_runtime_15_m3_row_data_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 test file budget root-layout status scan child split" {
        Some("runtime_15_test_file_budget_root_layout_status_scan_child_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 historical oversized test roots closeout" {
        Some("runtime_15_historical_oversized_test_roots_closeout_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 asset test-budget guard child-owner split" {
        Some("runtime_15_asset_test_budget_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI asset test folder split" {
        Some("runtime_15_ui_asset_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI asset MUI X web style test folder split" {
        Some("runtime_15_ui_asset_mui_web_mui_x_style_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI asset MUI web style test folder split" {
        Some("runtime_15_ui_asset_mui_web_style_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI taffy layout pass test folder split" {
        Some("runtime_15_ui_taffy_layout_pass_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI runtime window input pump test folder split" {
        Some("runtime_15_ui_runtime_window_input_pump_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI runtime window event ABI child folder split" {
        Some("runtime_15_ui_runtime_window_event_abi_children_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 test file budget root-layout UI child split" {
        Some("runtime_15_test_file_budget_root_layout_ui_child_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI widget text input keyboard test folder split" {
        Some("runtime_15_ui_widget_text_input_keyboard_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI focus navigation test folder split" {
        Some("runtime_15_ui_focus_navigation_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI runtime input manager test folder split" {
        Some("runtime_15_ui_runtime_input_manager_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI runtime input ownership test folder split" {
        Some(
            "runtime_15_ui_runtime_input_ownership_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 production file budget guard child-owner split" {
        Some("runtime_15_production_file_budget_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 status output variable evidence anchors" {
        Some("runtime_15_status_output_variable_evidence_anchors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 status output M3 row data child-owner split" {
        Some("runtime_15_status_output_m3_row_data_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 status output row-data guard child-owner split" {
        Some(
            "runtime_15_status_output_row_data_guard_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 status output expected-slice legacy child-owner split" {
        Some("runtime_15_status_output_expected_slice_legacy_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 status output expected-slice legacy group child-owner split" {
        Some("runtime_15_status_output_expected_slice_legacy_group_child_owner_split_static_passed_cargo_deferred")
    } else {
        None
    }
}

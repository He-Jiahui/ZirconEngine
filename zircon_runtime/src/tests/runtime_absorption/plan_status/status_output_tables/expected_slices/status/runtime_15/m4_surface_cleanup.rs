pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M4 core runtime service-list owner split" {
        Some("runtime_15_core_runtime_service_lists_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M4 RHI WGPU command validation render-state owner split" {
        Some(
            "runtime_15_rhi_wgpu_command_validation_render_state_split_static_passed_cargo_lock_blocked",
        )
    } else if slice == "Runtime 15 M4 RHI WGPU UI surface render/setup owner split" {
        Some(
            "runtime_15_rhi_wgpu_ui_surface_render_setup_owner_split_static_passed_cargo_timeout_no_result",
        )
    } else if slice == "Runtime 15 M4 RHI WGPU UI surface geometry test owner split" {
        Some(
            "runtime_15_rhi_wgpu_ui_surface_geometry_tests_owner_split_static_passed_cargo_timeout_no_result",
        )
    } else if slice == "Runtime 15 M4 RHI device handle owner split" {
        Some("runtime_15_rhi_device_handles_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 dynamic API session profile owner split" {
        Some("runtime_15_dynamic_api_session_profile_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 dynamic API session registry owner split" {
        Some("runtime_15_dynamic_api_session_registry_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 native host API adapter tests owner split" {
        Some("runtime_15_native_host_api_adapter_tests_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 material asset value/readiness helper owner split" {
        Some(
            "runtime_15_material_asset_value_readiness_owner_split_static_passed_cargo_timeout_no_result",
        )
    } else if slice == "Runtime 15 M4 material asset management record owner split" {
        Some("runtime_15_material_asset_management_record_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 asset artifact cache UI document owner split" {
        Some(
            "runtime_15_asset_artifact_cache_ui_documents_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M4 mesh asset management record owner split" {
        Some("runtime_15_mesh_asset_management_record_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 asset project scan/import source collection owner split" {
        Some(
            "runtime_15_asset_project_scan_import_sources_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M4 glTF labeled material subasset owner split" {
        Some("runtime_15_gltf_labeled_material_subasset_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 texture descriptor settings parser owner split" {
        Some(
            "runtime_15_texture_descriptor_settings_parser_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M4 scene world render light collection owner split" {
        Some("runtime_15_scene_world_render_lights_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 scene component lighting/post-process owner split" {
        Some(
            "runtime_15_scene_component_light_postprocess_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 render shader template assembly guard WGSL contracts split" {
        Some(
            "runtime_15_render_shader_template_assembly_guard_wgsl_contracts_split_static_passed_cargo_deferred",
        )
    } else if slice
        == "Runtime 15 M4 core runtime render-stats graph execution-resources owner split"
    {
        Some(
            "runtime_15_render_stats_graph_execution_resources_owner_split_static_passed_cargo_timeout_no_result",
        )
    } else if slice == "Runtime 15 M4 render-stats product diagnostics test owner split" {
        Some(
            "runtime_15_render_stats_product_diagnostics_tests_owner_split_static_passed_cargo_deferred_active_editor_lane",
        )
    } else if slice == "Runtime 15 M4 scene fixed light reflection write-field owner split" {
        Some(
            "runtime_15_scene_fixed_light_reflection_write_fields_owner_split_static_passed_cargo_lock_blocked",
        )
    } else if slice == "Runtime 15 M4 scene world property-access physics write owner split" {
        Some(
            "runtime_15_scene_world_property_access_physics_owner_split_static_passed_cargo_timeout_no_result",
        )
    } else if slice == "Runtime 15 M4 scene world property-access physics entry owner split" {
        Some(
            "runtime_15_scene_world_property_access_physics_entries_owner_split_static_passed_cargo_lock_blocked",
        )
    } else if slice == "Runtime 15 M4 scene world project I/O mesh owner split" {
        Some(
            "runtime_15_scene_world_project_io_mesh_owner_split_static_passed_cargo_timeout_no_result",
        )
    } else if slice == "Runtime 15 M4 UI text layout engine visual-order owner split" {
        Some(
            "runtime_15_ui_text_layout_engine_visual_order_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M4 UI layout arrange grid/masonry owner split" {
        Some("runtime_15_ui_layout_arrange_grid_masonry_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 UI dispatch input manager test owner split" {
        Some("runtime_15_ui_dispatch_input_manager_tests_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 UI template MUI X DataGrid class owner split" {
        Some(
            "runtime_15_ui_template_mui_x_data_grid_class_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M4 UI template document validation owner split" {
        Some("runtime_15_ui_template_document_validation_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 UI template style slot-contract owner split" {
        Some(
            "runtime_15_ui_template_style_slot_contract_owner_split_static_passed_cargo_timeout_no_result",
        )
    } else if slice == "Runtime 15 M4 UI v2 style runtime-state owner split" {
        Some("runtime_15_ui_v2_style_runtime_state_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 UI v2 style token-resolution owner split" {
        Some("runtime_15_ui_v2_style_token_resolution_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 UI accessibility extract state owner split" {
        Some("runtime_15_ui_accessibility_extract_state_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 UI component catalog editor-showcase helper owner split" {
        Some(
            "runtime_15_ui_component_catalog_editor_showcase_helper_owner_split_static_passed_cargo_timeout_no_result",
        )
    } else if slice == "Runtime 15 M4 UI component state-reducer keyboard menu submenu owner split"
    {
        Some(
            "runtime_15_ui_component_state_reducer_keyboard_menu_submenu_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M4 UI component state-reducer tree view editing owner split" {
        Some(
            "runtime_15_ui_component_state_reducer_tree_view_editing_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M4 UI surface event-routing owner split" {
        Some("runtime_15_ui_surface_event_routing_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M4 UI surface property mutation metadata dirty owner split" {
        Some(
            "runtime_15_ui_surface_property_mutation_metadata_dirty_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M4 UI surface render feedback command/color owner split" {
        Some(
            "runtime_15_ui_surface_render_feedback_command_color_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M4 UI surface default-interactions keyboard/timer owner split" {
        Some(
            "runtime_15_ui_surface_default_interactions_keyboard_timer_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M4 UI surface table column helper owner split" {
        Some("runtime_15_ui_surface_table_column_helper_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F12 offscreen target texture owner cleanup" {
        Some(
            "runtime_15_offscreen_target_texture_owner_cleanup_static_passed_cargo_timeout_no_result",
        )
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
        Some(
            "runtime_15_resource_streamer_diagnostics_accessor_cleanup_static_passed_cargo_lock_blocked",
        )
    } else if slice == "Runtime 15 F12 resource streamer resolve texture id cleanup" {
        Some(
            "runtime_15_resource_streamer_resolve_texture_id_cleanup_static_passed_cargo_lock_blocked",
        )
    } else if slice == "Runtime 15 F12 particle GPU readback output accessor cleanup" {
        Some(
            "runtime_15_particle_gpu_readback_output_accessor_cleanup_static_passed_cargo_lock_blocked",
        )
    } else if slice == "Runtime 15 F12 advanced plugin output test accessor cleanup" {
        Some(
            "runtime_15_advanced_plugin_output_test_accessor_cleanup_static_passed_cargo_lock_blocked",
        )
    } else {
        None
    }
}

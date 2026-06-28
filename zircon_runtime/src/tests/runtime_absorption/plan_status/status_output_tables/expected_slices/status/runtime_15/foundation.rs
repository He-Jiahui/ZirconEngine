pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 F9 runtime prelude required type coverage" {
        Some("runtime_15_prelude_required_types_coremin_check_passed")
    } else if slice == "Runtime 15 runtime UI dead-code support split" {
        Some("runtime_15_runtime_ui_dead_code_support_split_coremin_check_passed")
    } else if slice == "Runtime 15 M5 production dead-code suppression global gate" {
        Some("runtime_15_production_dead_code_suppression_global_gate_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F12 dead-code review status sync" {
        Some("runtime_15_f12_dead_code_review_status_sync_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F12 dead-code runtime/editor boundary status guard" {
        Some("runtime_15_f12_dead_code_runtime_editor_boundary_status_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F12 UI text edit-state dead-code suppression cleanup" {
        Some(
            "runtime_15_ui_text_edit_state_dead_code_suppression_cleanup_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 UI boundary runtime-host forbidden attribute literal cleanup" {
        Some("runtime_15_ui_boundary_runtime_host_literal_cleanup_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F1 native host callback panic guard" {
        Some("runtime_15_native_host_callback_panic_guard_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 lock poison policy guard folder split" {
        Some("runtime_15_lock_poison_policy_guard_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 core runtime lock poison guard child-owner split" {
        Some(
            "runtime_15_core_runtime_lock_poison_guard_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 F2 lock poison recovery guard" {
        Some("runtime_15_f2_lock_poison_recovery_guard_core_min_cargo_passed_full_sweep_pending")
    } else if slice == "Runtime 15 M3 production direct lock unwrap global gate" {
        Some("runtime_15_production_direct_lock_unwrap_global_gate_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 config store lock poison recovery" {
        Some("runtime_15_config_store_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 core runtime devtools lock poison recovery" {
        Some("runtime_15_core_runtime_devtools_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 core handle diagnostics lock poison recovery" {
        Some("runtime_15_core_handle_diagnostics_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 core handle time lock poison recovery" {
        Some("runtime_15_core_handle_time_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 core handle states lock poison recovery" {
        Some("runtime_15_core_handle_states_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 core runtime task lock poison recovery" {
        Some("runtime_15_core_runtime_task_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 core runtime profiling lock poison recovery" {
        Some("runtime_15_core_runtime_profiling_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 core handle registry lock poison recovery" {
        Some("runtime_15_core_handle_registry_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 core runtime registration structure behavior layout split" {
        Some(
            "runtime_15_core_runtime_registration_structure_behavior_layout_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 core runtime registration structure owner split" {
        Some(
            "runtime_15_core_runtime_registration_structure_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 root entries guard child-owner split" {
        Some("runtime_15_root_entries_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 plugin bridge table lock poison recovery" {
        Some("runtime_15_plugin_bridge_table_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 native live-host bridge methods lock poison recovery" {
        Some(
            "runtime_15_native_live_host_bridge_methods_lock_poison_recovery_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 navigation lock poison recovery" {
        Some("runtime_15_navigation_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 dynamic API session lock poison recovery" {
        Some("runtime_15_dynamic_api_session_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 dynamic scene spawn task lock poison recovery" {
        Some(
            "runtime_15_dynamic_scene_spawn_task_lock_poison_recovery_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 scene ECS parallel executor lock poison recovery" {
        Some(
            "runtime_15_scene_ecs_parallel_executor_lock_poison_recovery_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 core resource manager lock poison recovery" {
        Some("runtime_15_core_resource_manager_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 asset project manager lock poison recovery" {
        Some("runtime_15_asset_project_manager_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 asset worker pool lock poison recovery" {
        Some("runtime_15_asset_worker_pool_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 WGPU render framework lock poison recovery" {
        Some("runtime_15_wgpu_render_framework_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 RHI WGPU render device lock poison recovery" {
        Some("runtime_15_rhi_wgpu_render_device_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 animation manager lock poison recovery" {
        Some("runtime_15_animation_manager_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 input runtime manager lock poison recovery" {
        Some("runtime_15_input_runtime_manager_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 script VM registry lock poison recovery" {
        Some("runtime_15_script_vm_registry_lock_poison_recovery_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 ZrVM real backend runtime lock poison recovery" {
        Some(
            "runtime_15_zr_vm_real_backend_runtime_lock_poison_recovery_static_passed_cargo_timeout_no_result",
        )
    } else if slice == "Runtime 15 M3 VM plugin manager selected-backend lock poison recovery" {
        Some(
            "runtime_15_vm_plugin_manager_selected_backend_lock_poison_recovery_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 graphics facade visibility note" {
        Some(
            "runtime_15_graphics_facade_visibility_note_static_passed_cargo_blocked_graphics_drift",
        )
    } else if slice == "Runtime 15 F14 diagnostics normalization" {
        Some("runtime_15_diagnostics_frame_trait_wrapper_removed_coremin_check_passed")
    } else if slice == "Runtime 15 F5 scene property access typed errors" {
        Some("runtime_15_scene_property_access_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 animation manager typed errors" {
        Some("runtime_15_animation_manager_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 animation asset binary typed errors" {
        Some("runtime_15_animation_asset_binary_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 profile export typed errors" {
        Some("runtime_15_profile_export_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 gameplay host typed errors" {
        Some("runtime_15_gameplay_host_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 script scene hook typed errors" {
        Some("runtime_15_script_scene_hook_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 VM plugin management policy typed errors" {
        Some("runtime_15_vm_plugin_management_policy_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 UI surface input effect typed errors" {
        Some("runtime_15_ui_surface_input_effect_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 UI input surrounding-text error source" {
        Some("runtime_15_ui_input_surrounding_text_error_source_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 UI template resource resolver typed errors" {
        Some("runtime_15_ui_template_resource_resolver_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 UI asset document typed errors" {
        Some("runtime_15_ui_asset_document_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 export CLI typed errors" {
        Some("runtime_15_export_cli_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 host reflection docs CLI typed errors" {
        Some("runtime_15_host_reflection_docs_cli_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 dynamic API session typed errors" {
        Some("runtime_15_dynamic_api_session_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 typed API residual typed errors" {
        Some("runtime_15_typed_api_residual_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 fixed world mutation typed errors" {
        Some("runtime_15_fixed_world_mutation_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 asset authoring typed errors" {
        Some("runtime_15_asset_authoring_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 navigation asset typed errors" {
        Some("runtime_15_navigation_asset_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 font asset typed errors" {
        Some("runtime_15_font_asset_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 sound asset typed errors" {
        Some("runtime_15_sound_asset_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 zshader definition typed errors" {
        Some("runtime_15_zshader_definition_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 asset meta typed errors" {
        Some("runtime_15_asset_meta_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 texture loader typed errors" {
        Some("runtime_15_texture_loader_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 mesh loader typed errors" {
        Some("runtime_15_mesh_loader_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F8 texture descriptor typed errors" {
        Some("runtime_15_texture_descriptor_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F8 RuntimePluginDescriptor status mirror cleanup" {
        Some(
            "runtime_15_runtime_plugin_descriptor_status_mirror_cleanup_static_passed_cargo_deferred",
        )
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
        Some(
            "runtime_15_script_reflection_macro_fixture_dead_code_cleanup_static_passed_cargo_deferred",
        )
    } else {
        None
    }
}

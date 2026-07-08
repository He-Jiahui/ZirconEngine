pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M3 lock poison policy guard folder split" {
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
    } else if slice == "Runtime 15 M3 root entries module-families guard folder-backed split" {
        Some(
            "runtime_15_root_entries_module_families_guard_folder_backed_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 core spine root/generated route-owner split" {
        Some("runtime_15_core_spine_root_generated_route_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 core spine root/generated audit source sync" {
        Some("runtime_15_core_spine_root_generated_audit_source_sync_static_passed_cargo_deferred")
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
    } else {
        None
    }
}

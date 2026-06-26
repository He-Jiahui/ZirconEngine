use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 15 M3 graphics dead-code guard module split",
        &[
            "runtime_15_graphics_dead_code_guard_module_split_static_passed_cargo_lock_blocked",
            "graphics_dead_code/module_layout.rs",
            "graphics_dead_code/renderer_output_accessors.rs",
            "runtime_15_graphics_dead_code_guard_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 graphics dead-code guard child-owner split",
        &[
            "runtime_15_graphics_dead_code_guard_child_owner_split_static_passed_cargo_deferred",
            "graphics_dead_code/backend_owners.rs",
            "graphics_dead_code/gpu_resource_owners.rs",
            "graphics_dead_code/resource_streamer_cleanup.rs",
        ],
    ),
    (
        "Runtime 15 M3 provider boilerplate guard module split",
        &[
            "runtime_15_provider_boilerplate_guard_module_split_static_passed_cargo_lock_blocked",
            "structure_convention/provider_boilerplate.rs",
            "runtime_15_provider_boilerplate_guard_is_folder_backed",
            "runtime_15_provider_registration_uses_shared_owner",
        ],
    ),
    (
        "Runtime 15 M3 provider boilerplate guard child-owner split",
        &[
            "runtime_15_provider_boilerplate_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/provider_boilerplate.rs",
            "structure_convention/provider_boilerplate/module_layout.rs",
            "structure_convention/provider_boilerplate/full_audit.rs",
            "runtime_15_provider_boilerplate_guard_child_owner_split",
        ],
    ),
    (
        "Runtime 15 M3 facade surface guard module split",
        &[
            "runtime_15_facade_surface_guard_module_split_static_passed_cargo_lock_blocked",
            "structure_convention/facade_surface.rs",
            "runtime_15_facade_surface_guard_is_folder_backed",
            "runtime_15_prelude_covers_required_types",
        ],
    ),
    (
        "Runtime 15 M3 runtime dead-code guard module split",
        &[
            "runtime_15_runtime_dead_code_guard_module_split_static_passed_cargo_lock_blocked",
            "structure_convention/runtime_dead_code.rs",
            "runtime_15_runtime_dead_code_guard_is_folder_backed",
            "runtime_15_runtime_ui_dead_code_surface_is_test_support",
        ],
    ),
    (
        "Runtime 15 M3 runtime dead-code guard forbidden attribute literal cleanup",
        &[
            "runtime_15_runtime_dead_code_guard_literal_cleanup_static_passed_cargo_deferred",
            "structure_convention/runtime_dead_code.rs",
            "DEAD_CODE_ALLOW_ATTRIBUTE",
            "runtime_15_runtime_dead_code_guard_forbidden_attribute_literal_is_constant_backed",
        ],
    ),
    (
        "Runtime 15 M3 lock poison policy guard folder split",
        &[
            "runtime_15_lock_poison_policy_guard_folder_split_static_passed_cargo_deferred",
            "structure_convention/lock_poison_policy.rs",
            "structure_convention/lock_poison_policy/core_runtime.rs",
            "structure_convention/lock_poison_policy/runtime_services.rs",
            "structure_convention/lock_poison_policy/asset_render_input.rs",
            "runtime_15_lock_poison_policy_guard_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 core runtime lock poison guard child-owner split",
        &[
            "runtime_15_core_runtime_lock_poison_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/lock_poison_policy/core_runtime.rs",
            "structure_convention/lock_poison_policy/core_runtime/handle_accessors.rs",
            "runtime_15_core_runtime_lock_poison_guard_child_owner_split",
        ],
    ),
    (
        "Runtime 15 M3 F2 lock poison recovery guard",
        &[
            "runtime_15_f2_lock_poison_recovery_guard_static_passed_cargo_deferred",
            "structure_convention/lock_poison_policy.rs",
            "scene/level_system.rs",
            "core/runtime/events.rs",
            "runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus",
        ],
    ),
    (
        "Runtime 15 M3 production direct lock unwrap global gate",
        &[
            "runtime_15_production_direct_lock_unwrap_global_gate_static_passed_cargo_deferred",
            "structure_convention/lock_poison_policy/core_runtime.rs",
            "LOCK_UNWRAP_CALL",
            "runtime_15_production_sources_do_not_directly_unwrap_mutex_locks",
        ],
    ),
    (
        "Runtime 15 M3 config store lock poison recovery",
        &[
            "runtime_15_config_store_lock_poison_recovery_static_passed_cargo_deferred",
            "core/runtime/config_store.rs",
            "docs/zircon_runtime/core/runtime/config_store.md",
            "runtime_15_config_store_lock_poison_recovery_guard_covers_runtime_config_store",
        ],
    ),
    (
        "Runtime 15 M3 core runtime devtools lock poison recovery",
        &[
            "runtime_15_core_runtime_devtools_lock_poison_recovery_static_passed_cargo_deferred",
            "core/runtime/diagnostics/devtools.rs",
            "devtools_snapshot_recovers_poisoned_runtime_registry_locks",
            "runtime_15_core_runtime_devtools_lock_poison_recovery_guard_covers_devtools_snapshot",
        ],
    ),
    (
        "Runtime 15 M3 core handle diagnostics lock poison recovery",
        &[
            "runtime_15_core_handle_diagnostics_lock_poison_recovery_static_passed_cargo_deferred",
            "core/runtime/handle/diagnostics.rs",
            "core_handle_diagnostic_accessors_recover_poisoned_store_lock",
            "runtime_15_core_handle_diagnostics_lock_poison_recovery_guard_covers_diagnostic_store",
        ],
    ),
    (
        "Runtime 15 M3 core handle time lock poison recovery",
        &[
            "runtime_15_core_handle_time_lock_poison_recovery_static_passed_cargo_deferred",
            "core/runtime/handle/time.rs",
            "core_handle_time_accessors_recover_poisoned_runtime_clocks",
            "runtime_15_core_handle_time_lock_poison_recovery_guard_covers_runtime_clocks",
        ],
    ),
    (
        "Runtime 15 M3 core handle states lock poison recovery",
        &[
            "runtime_15_core_handle_states_lock_poison_recovery_static_passed_cargo_deferred",
            "core/runtime/handle/states.rs",
            "core_handle_state_accessors_recover_poisoned_state_registry_lock",
            "runtime_15_core_handle_states_lock_poison_recovery_guard_covers_state_registry",
        ],
    ),
    (
        "Runtime 15 M3 core runtime task lock poison recovery",
        &[
            "runtime_15_core_runtime_task_lock_poison_recovery_static_passed_cargo_deferred",
            "core/runtime/tasks/job_handle.rs",
            "core/runtime/tasks/job_scheduler.rs",
            "job_handle_accessors_recover_poisoned_state_lock",
            "pending_scheduled_job_recovers_poisoned_task_lock",
            "runtime_15_core_runtime_task_lock_poison_recovery_guard_covers_job_handles",
        ],
    ),
    (
        "Runtime 15 M3 core runtime profiling lock poison recovery",
        &[
            "runtime_15_core_runtime_profiling_lock_poison_recovery_static_passed_cargo_deferred",
            "core/runtime/diagnostics/profiling/mod.rs",
            "profile_recorder_accessors_recover_poisoned_global_lock",
            "runtime_15_core_runtime_profiling_lock_poison_recovery_guard_covers_global_recorder",
        ],
    ),
    (
        "Runtime 15 M3 core handle registry lock poison recovery",
        &[
            "runtime_15_core_handle_registry_lock_poison_recovery_static_passed_cargo_deferred",
            "core/runtime/handle/core_handle.rs",
            "core/runtime/handle/registration/register_module.rs",
            "core_handle_registry_accessors_recover_poisoned_runtime_locks",
            "runtime_15_core_handle_registry_lock_poison_recovery_guard_covers_registry_accessors",
        ],
    ),
    (
        "Runtime 15 M3 core runtime registration structure behavior layout split",
        &[
            "runtime_15_core_runtime_registration_structure_behavior_layout_split_static_passed_cargo_deferred",
            "core/runtime/tests/registration/structure.rs",
            "core/runtime/tests/registration/structure/behavior_layout.rs",
            "registration_behavior_tests_stay_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 core runtime registration structure owner split",
        &[
            "runtime_15_core_runtime_registration_structure_owner_split_static_passed_cargo_deferred",
            "core/runtime/tests/registration/structure/mod.rs",
            "core/runtime/tests/registration/structure/service_count_paths.rs",
            "core/runtime/tests/registration/structure/service_list_caches.rs",
            "runtime_15_core_runtime_registration_structure_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 root entries guard child-owner split",
        &[
            "runtime_15_root_entries_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/root_entries.rs",
            "tests/runtime_absorption/root_entries/core_spine.rs",
            "tests/runtime_absorption/root_entries/module_families.rs",
            "tests/runtime_absorption/root_entries/runtime_root.rs",
            "runtime_15_root_entries_guard_child_owners_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 plugin bridge table lock poison recovery",
        &[
            "runtime_15_plugin_bridge_table_lock_poison_recovery_static_passed_cargo_deferred",
            "plugin/bridge/table.rs",
            "docs/zircon_runtime/plugin/bridge.md",
            "bridge_entry_provider_accessors_recover_poisoned_provider_lock",
            "runtime_15_plugin_bridge_table_lock_poison_recovery_guard_covers_provider_slot",
        ],
    ),
    (
        "Runtime 15 M3 native live-host bridge methods lock poison recovery",
        &[
            "runtime_15_native_live_host_bridge_methods_lock_poison_recovery_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs",
            "docs/zircon_runtime/plugin/bridge.md",
            "native_live_host_bridge_method_bindings_recover_poisoned_lock",
            "runtime_15_native_live_host_bridge_methods_lock_poison_recovery_guard_covers_binding_registry",
        ],
    ),
    (
        "Runtime 15 M3 navigation lock poison recovery",
        &[
            "runtime_15_navigation_lock_poison_recovery_static_passed_cargo_deferred",
            "navigation/runtime.rs",
            "docs/zircon_runtime/navigation/runtime.md",
            "runtime_15_navigation_lock_poison_recovery_guard_covers_builtin_navigation_manager",
        ],
    ),
    (
        "Runtime 15 M3 dynamic API session lock poison recovery",
        &[
            "runtime_15_dynamic_api_session_lock_poison_recovery_static_passed_cargo_deferred",
            "dynamic_api/session.rs",
            "dynamic_api/session/tests/lock_poison.rs",
            "runtime_15_dynamic_api_session_lock_poison_recovery_guard_covers_session_registry",
        ],
    ),
    (
        "Runtime 15 M3 dynamic scene spawn task lock poison recovery",
        &[
            "runtime_15_dynamic_scene_spawn_task_lock_poison_recovery_static_passed_cargo_deferred",
            "scene/dynamic_scene/spawn_task/task.rs",
            "scene/dynamic_scene/spawn_task/loader.rs",
            "dynamic_scene_spawn_task_accessors_recover_poisoned_locks",
            "runtime_15_dynamic_scene_spawn_task_lock_poison_recovery_guard_covers_spawn_task",
        ],
    ),
    (
        "Runtime 15 M3 scene ECS parallel executor lock poison recovery",
        &[
            "runtime_15_scene_ecs_parallel_executor_lock_poison_recovery_static_passed_cargo_deferred",
            "scene/ecs/schedule_parallel_executor.rs",
            "docs/zircon_runtime/scene/ecs.md",
            "schedule_parallel_executor_batch_result_slot_recovers_poisoned_lock",
            "runtime_15_scene_ecs_parallel_executor_lock_poison_recovery_guard_covers_batch_result_slots",
        ],
    ),
    (
        "Runtime 15 M3 core resource manager lock poison recovery",
        &[
            "runtime_15_core_resource_manager_lock_poison_recovery_static_passed_cargo_deferred",
            "core/resource/manager/resource_manager.rs",
            "core/resource/manager/registry_ops.rs",
            "runtime_15_core_resource_manager_lock_poison_recovery_guard_covers_resource_manager",
        ],
    ),
    (
        "Runtime 15 M3 asset project manager lock poison recovery",
        &[
            "runtime_15_asset_project_manager_lock_poison_recovery_static_passed_cargo_deferred",
            "asset/pipeline/manager/project_asset_manager/runtime.rs",
            "asset/pipeline/manager/project_asset_manager/construction.rs",
            "project_asset_manager_runtime_accessors_recover_poisoned_locks",
            "runtime_15_asset_project_manager_lock_poison_recovery_guard_covers_project_asset_manager",
        ],
    ),
    (
        "Runtime 15 M3 asset worker pool lock poison recovery",
        &[
            "runtime_15_asset_worker_pool_lock_poison_recovery_static_passed_cargo_deferred",
            "asset/pipeline/worker_pool.rs",
            "asset/pipeline/manager/service_contracts/asset_manager_contract.rs",
            "asset_worker_pool_accessors_recover_poisoned_locks",
            "runtime_15_asset_worker_pool_lock_poison_recovery_guard_covers_asset_worker_pool",
        ],
    ),
    (
        "Runtime 15 M3 WGPU render framework lock poison recovery",
        &[
            "runtime_15_wgpu_render_framework_lock_poison_recovery_static_passed_cargo_deferred",
            "graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs",
            "docs/zircon_runtime/graphics/render-product-submit.md",
            "wgpu_render_framework_accessors_recover_poisoned_locks",
            "runtime_15_wgpu_render_framework_lock_poison_recovery_guard_covers_wgpu_framework",
        ],
    ),
    (
        "Runtime 15 M3 RHI WGPU render device lock poison recovery",
        &[
            "runtime_15_rhi_wgpu_render_device_lock_poison_recovery_static_passed_cargo_deferred",
            "rhi_wgpu/device.rs",
            "docs/zircon_runtime/rhi/descriptors.md",
            "wgpu_render_device_state_accessors_recover_poisoned_lock",
            "runtime_15_rhi_wgpu_render_device_lock_poison_recovery_guard_covers_device_state",
        ],
    ),
    (
        "Runtime 15 M3 animation manager lock poison recovery",
        &[
            "runtime_15_animation_manager_lock_poison_recovery_static_passed_cargo_deferred",
            "animation/manager/mod.rs",
            "animation_manager_playback_settings_recover_poisoned_lock",
            "runtime_15_animation_manager_lock_poison_recovery_guard_covers_playback_settings",
        ],
    ),
    (
        "Runtime 15 M3 input runtime manager lock poison recovery",
        &[
            "runtime_15_input_runtime_manager_lock_poison_recovery_static_passed_cargo_deferred",
            "input/runtime/default_input_manager.rs",
            "input/runtime/default_input_action_manager.rs",
            "input_manager_accessors_recover_poisoned_state_lock",
            "runtime_15_input_runtime_manager_lock_poison_recovery_guard_covers_input_state",
        ],
    ),
    (
        "Runtime 15 M3 script VM registry lock poison recovery",
        &[
            "runtime_15_script_vm_registry_lock_poison_recovery_static_passed_cargo_deferred",
            "script/vm/backend/backend_registry.rs",
            "script/vm/host/host_export_registry.rs",
            "script/vm/runtime/hot_reload_coordinator.rs",
            "hot_reload_coordinator_accessors_recover_poisoned_slot_table_lock",
            "runtime_15_script_vm_registry_lock_poison_recovery_guard_covers_vm_registries",
        ],
    ),
    (
        "Runtime 15 M3 VM plugin manager selected-backend lock poison recovery",
        &[
            "runtime_15_vm_plugin_manager_selected_backend_lock_poison_recovery_static_passed_cargo_deferred",
            "script/vm/runtime/vm_plugin_manager.rs",
            "docs/zircon_runtime/script/vm/zr_vm_host_reflection.md",
            "vm_plugin_manager_selected_backend_accessors_recover_poisoned_lock",
            "runtime_15_vm_plugin_manager_selected_backend_lock_poison_recovery_guard_covers_manager_selector",
        ],
    ),
    (
        "Runtime 15 M3 diagnostics guard module split",
        &[
            "runtime_15_diagnostics_guard_module_split_static_passed_cargo_lock_blocked",
            "structure_convention/diagnostics_surface.rs",
            "runtime_15_diagnostics_guard_is_folder_backed",
            "runtime_15_diagnostics_use_frame_trait_without_world_wrapper",
        ],
    ),
    (
        "Runtime 15 M3 core framework test folder split",
        &[
            "runtime_15_core_framework_tests_folder_split_static_passed_cargo_lock_blocked",
            "core/framework/tests/framework_surfaces.rs",
            "core/framework/tests/render_product_surface.rs",
            "runtime_15_core_framework_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 picking test folder split",
        &[
            "runtime_15_picking_tests_folder_split_static_passed_cargo_deferred",
            "tests/picking/mod.rs",
            "tests/picking/rays.rs",
            "tests/picking/diagnostics.rs",
            "tests/picking/pipeline.rs",
            "tests/picking/pointer_events.rs",
            "runtime_15_picking_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 core runtime deactivation blocked test folder split",
        &[
            "runtime_15_core_runtime_deactivation_blocked_tests_folder_split_static_passed_cargo_deferred",
            "core/runtime/tests/activation/behavior/deactivation/blocked.rs",
            "core/runtime/tests/activation/behavior/deactivation/blocked/external_dependents.rs",
            "core/runtime/tests/activation/behavior/deactivation/blocked/exact_four_dependency_matcher.rs",
            "runtime_15_core_runtime_deactivation_blocked_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 code review findings test folder split",
        &[
            "runtime_15_code_review_findings_tests_folder_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/code_review_findings.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
            "tests/runtime_absorption/code_review_findings/f8_api_convergence.rs",
            "runtime_15_code_review_findings_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 typed-error convergence guard child-owner split",
        &[
            "runtime_15_typed_error_convergence_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world.rs",
            "review_f5_sound_asset_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 M3 dynamic scene absorption guard folder split",
        &[
            "runtime_15_dynamic_scene_absorption_guard_folder_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/dynamic_scene.rs",
            "tests/runtime_absorption/dynamic_scene/patch_preview_api.rs",
            "tests/runtime_absorption/dynamic_scene/asset_reload_selection_status.rs",
            "runtime_15_dynamic_scene_absorption_guard_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 input manager test folder split",
        &[
            "runtime_15_input_manager_tests_folder_split_static_passed_cargo_deferred",
            "input/tests/input_manager.rs",
            "input/tests/input_manager/frame_state.rs",
            "input/tests/input_manager/touch_gamepad.rs",
            "runtime_15_input_manager_tests_are_folder_backed",
        ],
    ),
];

type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M4 no oversized production files global gate",
        &[
            "runtime_15_no_oversized_production_files_global_gate_static_passed_cargo_deferred",
            "structure_convention/production_file_budget/global_budget.rs",
            "PRODUCTION_FILE_LINE_BUDGET",
            "runtime_15_no_oversized_production_files",
        ],
    ),
    (
        "Runtime 15 M4 core runtime service-list owner split",
        &[
            "runtime_15_core_runtime_service_lists_folder_split_static_passed_cargo_lock_blocked",
            "core/runtime/handle/registration/service_lists/mod.rs",
            "core/runtime/handle/registration/service_lists/specialized.rs",
            "runtime_15_core_runtime_service_lists_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M4 RHI WGPU command validation render-state owner split",
        &[
            "runtime_15_rhi_wgpu_command_validation_render_state_split_static_passed_cargo_lock_blocked",
            "rhi_wgpu/command_validation.rs",
            "rhi_wgpu/command_validation/render_state.rs",
            "runtime_15_rhi_wgpu_command_validation_state_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 RHI WGPU device command-list owner split",
        &[
            "runtime_15_rhi_wgpu_device_command_list_owner_split_static_passed_cargo_deferred",
            "rhi_wgpu/device.rs",
            "rhi_wgpu/device/command_list.rs",
            "runtime_15_rhi_wgpu_device_command_list_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 RHI WGPU UI surface render/setup owner split",
        &[
            "runtime_15_rhi_wgpu_ui_surface_render_setup_owner_split_static_passed_cargo_timeout_no_result",
            "rhi_wgpu/ui_surface.rs",
            "rhi_wgpu/ui_surface/render_pass.rs",
            "runtime_15_rhi_wgpu_ui_surface_render_setup_are_child_owners",
        ],
    ),
    (
        "Runtime 15 M4 RHI WGPU UI surface geometry test owner split",
        &[
            "runtime_15_rhi_wgpu_ui_surface_geometry_tests_owner_split_static_passed_cargo_timeout_no_result",
            "rhi_wgpu/ui_surface/geometry.rs",
            "rhi_wgpu/ui_surface/geometry/tests.rs",
            "runtime_15_rhi_wgpu_ui_surface_geometry_tests_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 RHI device handle owner split",
        &[
            "runtime_15_rhi_device_handles_owner_split_static_passed_cargo_deferred",
            "rhi/device.rs",
            "rhi/device/handles.rs",
            "runtime_15_rhi_device_handles_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 dynamic API session profile owner split",
        &[
            "runtime_15_dynamic_api_session_profile_owner_split_static_passed_cargo_deferred",
            "dynamic_api/session.rs",
            "dynamic_api/session/profile.rs",
            "runtime_15_dynamic_api_session_profile_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 dynamic API session registry owner split",
        &[
            "runtime_15_dynamic_api_session_registry_owner_split_static_passed_cargo_deferred",
            "dynamic_api/session.rs",
            "dynamic_api/session/registry.rs",
            "runtime_15_dynamic_api_session_registry_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 dynamic API shader prewarm tests owner split",
        &[
            "runtime_15_dynamic_api_shader_prewarm_tests_owner_split_static_passed_cargo_deferred",
            "dynamic_api/shader_prewarm.rs",
            "dynamic_api/shader_prewarm/tests.rs",
            "runtime_15_dynamic_api_shader_prewarm_tests_are_child_owner",
        ],
    ),
];

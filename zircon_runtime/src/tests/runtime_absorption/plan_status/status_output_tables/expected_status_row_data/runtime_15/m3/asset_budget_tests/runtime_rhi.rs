type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 runtime diagnostics test folder split",
        &[
            "runtime_15_runtime_diagnostics_tests_folder_split_static_passed_cargo_lock_blocked",
            "tests/runtime_diagnostics/mod.rs",
            "tests/runtime_diagnostics/graph_resources.rs",
            "tests/runtime_diagnostics/gpu_sprite_ui_advanced.rs",
            "runtime_15_runtime_diagnostics_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 RHI command list test folder split",
        &[
            "runtime_15_rhi_command_list_tests_folder_split_static_passed_cargo_lock_blocked",
            "rhi/tests/command_list.rs",
            "rhi/tests/command_list/basic_commands.rs",
            "rhi/tests/command_list/vertex_index_state.rs",
            "runtime_15_rhi_command_list_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 RHI device contract test folder split",
        &[
            "runtime_15_rhi_device_contract_tests_folder_split_static_passed_cargo_lock_blocked",
            "rhi/tests/device_contract.rs",
            "rhi/tests/device_contract/basic_resources.rs",
            "rhi/tests/device_contract/framework_boundary.rs",
            "runtime_15_rhi_device_contract_tests_are_folder_backed",
        ],
    ),
];

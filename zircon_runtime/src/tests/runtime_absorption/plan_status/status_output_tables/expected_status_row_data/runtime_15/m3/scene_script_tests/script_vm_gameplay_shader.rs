type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 gameplay host test folder split",
        &[
            "runtime_15_gameplay_host_tests_folder_split_static_passed_cargo_deferred",
            "script/vm/gameplay_host/tests.rs",
            "script/vm/gameplay_host/tests/spawn_transform.rs",
            "script/vm/gameplay_host/tests/property_animation.rs",
            "runtime_15_gameplay_host_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 script VM gameplay host guard child-owner split",
        &[
            "runtime_15_script_vm_gameplay_host_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests/gameplay_host.rs",
            "script/vm/gameplay_host/tests.rs",
            "script/vm/gameplay_host/tests/spawn_transform.rs",
            "script/vm/gameplay_host/tests/property_animation.rs",
            "runtime_15_gameplay_host_tests_are_folder_backed",
            "runtime_15_script_vm_gameplay_host_guard_is_child_owner",
            "9 gameplay host tests",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 shader prewarm manifest test folder split",
        &[
            "runtime_15_shader_prewarm_manifest_tests_folder_split_static_passed_cargo_deferred",
            "bin/zircon_shader_prewarm/manifest.rs",
            "bin/zircon_shader_prewarm/manifest/tests.rs",
            "structure_convention/test_file_budget/shader_prewarm_manifest.rs",
            "runtime_15_shader_prewarm_manifest_tests_are_folder_backed",
        ],
    ),
];

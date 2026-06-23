use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
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
    (
        "Runtime 15 M3 asset pack test folder split",
        &[
            "runtime_15_asset_pack_tests_folder_split_static_passed_cargo_lock_blocked",
            "asset/tests/pack.rs",
            "asset/tests/pack/delta_installer.rs",
            "runtime_15_asset_pack_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset facade test folder split",
        &[
            "runtime_15_asset_facade_tests_folder_split_static_passed_cargo_lock_blocked",
            "asset/tests/facade.rs",
            "asset/tests/facade/recursive_dependencies.rs",
            "runtime_15_asset_facade_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset project zmeta test folder split",
        &[
            "runtime_15_asset_project_zmeta_tests_folder_split_static_passed_cargo_lock_blocked",
            "asset/tests/project/zmeta.rs",
            "asset/tests/project/zmeta/compound_shader.rs",
            "runtime_15_asset_project_zmeta_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset project manager test folder split",
        &[
            "runtime_15_asset_project_manager_tests_folder_split_static_passed_cargo_lock_blocked",
            "asset/tests/project/manager.rs",
            "asset/tests/project/manager/restore_failure_migration.rs",
            "runtime_15_asset_project_manager_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset project flow sample test folder split",
        &[
            "runtime_15_asset_project_flow_sample_tests_folder_split_static_passed_cargo_lock_blocked",
            "asset/tests/project/asset_flow_sample.rs",
            "asset/tests/project/asset_flow_sample/end_to_end.rs",
            "runtime_15_asset_project_flow_sample_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset material test folder split",
        &[
            "runtime_15_asset_material_tests_folder_split_static_passed_cargo_lock_blocked",
            "asset/tests/assets/material.rs",
            "asset/tests/assets/material/owned_descriptor.rs",
            "runtime_15_asset_material_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset glTF importer test folder split",
        &[
            "runtime_15_asset_gltf_importer_tests_folder_split_static_passed_cargo_lock_blocked",
            "asset/tests/assets/gltf_importer.rs",
            "asset/tests/assets/gltf_importer/labeled_subassets.rs",
            "runtime_15_asset_gltf_importer_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset glTF primitive fixture folder split",
        &[
            "runtime_15_asset_gltf_primitive_fixtures_folder_split_static_passed_cargo_lock_blocked",
            "asset/tests/assets/gltf_primitive_fixtures.rs",
            "asset/tests/assets/gltf_primitive_fixtures/vertex_channels.rs",
            "runtime_15_asset_gltf_primitive_fixtures_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset importer test folder split",
        &[
            "runtime_15_asset_importer_tests_folder_split_static_passed_cargo_lock_blocked",
            "asset/tests/assets/importer.rs",
            "asset/tests/assets/importer/typed_toml_ui.rs",
            "runtime_15_asset_importer_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset scene test folder split",
        &[
            "runtime_15_asset_scene_tests_folder_split_static_passed_cargo_lock_blocked",
            "asset/tests/assets/scene.rs",
            "asset/tests/assets/scene/foundation.rs",
            "runtime_15_asset_scene_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 test file budget guard folder split",
        &[
            "runtime_15_test_file_budget_guard_folder_split_static_passed_cargo_lock_blocked",
            "structure_convention/test_file_budget/mod.rs",
            "structure_convention/test_file_budget/runtime_diagnostics.rs",
            "structure_convention/test_file_budget/rhi_device_contract.rs",
            "structure_convention/test_file_budget/asset_gltf_primitive_fixtures.rs",
            "structure_convention/test_file_budget/asset_project_flow_sample.rs",
            "structure_convention/test_file_budget/asset_scene.rs",
            "structure_convention/test_file_budget/script_vm_tests.rs",
            "runtime_15_test_file_budget_guard_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 test file budget guard root mod cutover",
        &[
            "runtime_15_test_file_budget_guard_root_mod_cutover_static_passed_cargo_lock_blocked",
            "structure_convention/test_file_budget/mod.rs",
            "structure_convention/test_file_budget/root_layout.rs",
            "runtime_15_test_file_budget_guard_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset test-budget guard child-owner split",
        &[
            "runtime_15_asset_test_budget_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/asset_tests.rs",
            "structure_convention/test_file_budget/asset_tests/pack.rs",
            "structure_convention/test_file_budget/asset_tests/project.rs",
            "runtime_15_asset_test_budget_guard_child_owner_split",
        ],
    ),
];

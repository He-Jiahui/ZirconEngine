type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
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
        "Runtime 15 M3 no oversized test files global gate",
        &[
            "runtime_15_no_oversized_test_files_global_gate_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/global_budget.rs",
            "TEST_FILE_LINE_BUDGET",
            "runtime_15_no_oversized_test_files",
        ],
    ),
    (
        "Runtime 15 M3 render product mesh-cache morph tests child-owner split",
        &[
            "runtime_15_render_product_mesh_cache_morph_tests_child_owner_split_static_passed_cargo_deferred",
            "graphics/tests/render_product_mesh_cache/morph.rs",
            "graphics/tests/render_product_mesh_cache/morph/direct_velocity.rs",
            "graphics/tests/render_product_mesh_cache/morph/skinned_velocity.rs",
            "runtime_15_render_product_mesh_cache_morph_tests_are_child_owners",
        ],
    ),
    (
        "Runtime 15 M3 UI text layout edit-state test child-owner split",
        &[
            "runtime_15_ui_text_layout_edit_state_tests_child_owner_split_static_passed_cargo_deferred",
            "ui/tests/text_layout.rs",
            "ui/tests/text_layout/edit_state.rs",
            "runtime_15_ui_text_layout_edit_state_tests_are_child_owner",
        ],
    ),
];

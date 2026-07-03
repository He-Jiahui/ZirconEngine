type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 typed-error convergence guard child-owner split",
        &[
            "runtime_15_typed_error_convergence_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/diagnostics.rs",
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

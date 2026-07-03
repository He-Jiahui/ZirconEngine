type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 scene world typed-error review guard child-owner split",
        &[
            "runtime_15_scene_world_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world/typed_mutation_surface.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world/fixed_mutation.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world/dynamic_components.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world/property_access.rs",
            "review_f5_world_spawn_bundle_surface_uses_scene_error",
            "review_f5_scene_property_access_uses_scene_error",
            "runtime_15_code_review_findings_tests_are_folder_backed",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 script host typed-error review guard child-owner split",
        &[
            "runtime_15_script_host_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/script_host.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/script_host/gameplay_scene.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/script_host/plugin_management.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/script_host/host_reflection_docs.rs",
            "review_f5_gameplay_host_uses_typed_errors_before_script_host_boundary",
            "review_f5_host_reflection_docs_cli_uses_typed_errors_before_cli_boundary",
            "runtime_15_code_review_findings_tests_are_folder_backed",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 UI input typed-error review guard child-owner split",
        &[
            "runtime_15_ui_input_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input/surface_effects.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input/surrounding_text.rs",
            "review_f5_ui_surface_input_effects_use_typed_errors_before_rejected_reason_boundary",
            "review_f5_ui_input_surrounding_text_error_implements_std_error",
            "runtime_15_code_review_findings_typed_error_structure_guard_is_child_owner",
            "Cargo gate deferred",
        ],
    ),
];

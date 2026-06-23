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
            "tests/runtime_absorption/code_review_findings/typed_error_convergence.rs",
            "tests/runtime_absorption/code_review_findings/f8_api_convergence.rs",
            "runtime_15_code_review_findings_tests_are_folder_backed",
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
];

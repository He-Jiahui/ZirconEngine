type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
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
];

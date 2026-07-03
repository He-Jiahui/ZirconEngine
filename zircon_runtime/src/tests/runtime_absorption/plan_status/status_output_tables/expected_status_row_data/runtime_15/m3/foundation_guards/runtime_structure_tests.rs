type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
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

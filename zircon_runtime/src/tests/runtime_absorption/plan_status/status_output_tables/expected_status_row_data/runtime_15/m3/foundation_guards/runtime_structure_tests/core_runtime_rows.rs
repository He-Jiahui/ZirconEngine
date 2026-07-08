type Slice = super::Slice;

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
];

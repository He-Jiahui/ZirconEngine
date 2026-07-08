type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 plugin static manifest naming guard child-owner split",
        &[
            "runtime_15_plugin_static_manifest_naming_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/plugin_static_manifest.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/plugin_static_manifest/contract_owners.rs",
            "runtime_15_plugin_static_manifest_naming_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 banned-name scene-dynamic guard child-owner split",
        &[
            "runtime_15_banned_names_scene_dynamic_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names/scene_dynamic.rs",
            "runtime_15_banned_names_scene_dynamic_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 banned-name graphics construction guard child-owner split",
        &[
            "runtime_15_banned_names_graphics_construction_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names/graphics_construction.rs",
            "runtime_15_banned_names_graphics_construction_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 banned-name global module guard child-owner split",
        &[
            "runtime_15_banned_names_global_module_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names/global_modules.rs",
            "runtime_15_banned_names_global_module_guard_is_child_owner",
        ],
    ),
];

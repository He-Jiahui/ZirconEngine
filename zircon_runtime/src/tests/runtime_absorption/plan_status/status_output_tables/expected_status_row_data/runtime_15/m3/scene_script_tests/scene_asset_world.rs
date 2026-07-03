type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 dynamic scene root test folder split",
        &[
            "runtime_15_dynamic_scene_root_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/dynamic_scene.rs",
            "scene/tests/dynamic_scene/archive_manifest.rs",
            "scene/tests/dynamic_scene/scene_patch_document.rs",
            "runtime_15_dynamic_scene_root_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 scene render extract test folder split",
        &[
            "runtime_15_scene_render_extract_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/render_extract.rs",
            "scene/tests/render_extract/direct_sections.rs",
            "scene/tests/render_extract/lighting_postprocess.rs",
            "runtime_15_scene_render_extract_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 scene asset integration test folder split",
        &[
            "runtime_15_scene_asset_integration_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/asset_scene.rs",
            "scene/tests/asset_scene/mesh_bindings.rs",
            "scene/tests/asset_scene/product_fields.rs",
            "runtime_15_scene_asset_integration_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 scene world basics test folder split",
        &[
            "runtime_15_scene_world_basics_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/world_basics.rs",
            "scene/tests/world_basics/render_extract.rs",
            "scene/tests/world_basics/sprites.rs",
            "runtime_15_scene_world_basics_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 scene property paths test folder split",
        &[
            "runtime_15_scene_property_paths_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/property_paths.rs",
            "scene/tests/property_paths/read_paths.rs",
            "scene/tests/property_paths/runtime_mutation.rs",
            "scene/tests/property_paths/write_validation.rs",
            "runtime_15_scene_property_paths_tests_are_folder_backed",
        ],
    ),
];

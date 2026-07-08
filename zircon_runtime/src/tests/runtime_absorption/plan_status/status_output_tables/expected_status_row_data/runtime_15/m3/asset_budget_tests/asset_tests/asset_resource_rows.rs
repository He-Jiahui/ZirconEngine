type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 asset artifact store test folder split",
        &[
            "runtime_15_asset_artifact_store_tests_folder_split_static_passed_cargo_deferred",
            "asset/tests/assets/artifact_store.rs",
            "asset/tests/assets/artifact_store/binary_payloads.rs",
            "asset/tests/assets/artifact_store/library_assets.rs",
            "runtime_15_asset_artifact_store_tests_are_folder_backed",
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
        "Runtime 15 M3 asset mesh test root split",
        &[
            "runtime_15_asset_mesh_tests_root_split_static_passed_cargo_deferred",
            "asset/tests/assets/mesh.rs",
            "asset/tests/assets/mesh/document_roundtrip.rs",
            "asset/tests/assets/mesh/conversion_import.rs",
            "runtime_15_asset_mesh_tests_are_folder_backed",
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
];

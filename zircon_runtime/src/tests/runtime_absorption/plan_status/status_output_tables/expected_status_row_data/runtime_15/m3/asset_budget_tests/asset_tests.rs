type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 asset pack test folder split",
        &[
            "runtime_15_asset_pack_tests_folder_split_static_passed_cargo_lock_blocked",
            "asset/tests/pack.rs",
            "asset/tests/pack/delta_installer.rs",
            "runtime_15_asset_pack_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset pack panic-free header readers",
        &[
            "runtime_15_asset_pack_header_readers_panic_free_static_passed_cargo_deferred",
            "asset/pack/reader.rs",
            "asset/pack/delta.rs",
            "read_header_u64",
            "runtime_15_asset_pack_header_readers_are_panic_free",
        ],
    ),
    (
        "Runtime 15 M3 asset facade test folder split",
        &[
            "runtime_15_asset_facade_tests_folder_split_static_passed_cargo_lock_blocked",
            "asset/tests/facade.rs",
            "asset/tests/facade/recursive_dependencies.rs",
            "runtime_15_asset_facade_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset project zmeta test folder split",
        &[
            "runtime_15_asset_project_zmeta_tests_folder_split_static_passed_cargo_lock_blocked",
            "asset/tests/project/zmeta.rs",
            "asset/tests/project/zmeta/compound_shader.rs",
            "runtime_15_asset_project_zmeta_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset project zmeta current 12-test guard sync",
        &[
            "runtime_15_asset_project_zmeta_current_12_test_guard_sync_static_passed_cargo_deferred",
            "asset/tests/project/zmeta.rs",
            "asset/tests/project/zmeta/compound_shader.rs",
            "project_manager_derives_include_shader_import_path_from_project_and_package_path",
            "project_manager_reports_duplicate_shader_import_path_conflicts",
            "runtime_15_asset_project_zmeta_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset project manager test folder split",
        &[
            "runtime_15_asset_project_manager_tests_folder_split_static_passed_cargo_lock_blocked",
            "asset/tests/project/manager.rs",
            "asset/tests/project/manager/restore_failure_migration.rs",
            "runtime_15_asset_project_manager_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset project manager current 11-test guard sync",
        &[
            "runtime_15_asset_project_manager_current_11_test_guard_sync_static_passed_cargo_deferred",
            "asset/tests/project/manager.rs",
            "asset/tests/project/manager/restore_failure_migration.rs",
            "project_manager_records_import_dependency_ids_and_missing_dependency_diagnostics",
            "runtime_15_asset_project_manager_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset project flow sample test folder split",
        &[
            "runtime_15_asset_project_flow_sample_tests_folder_split_static_passed_cargo_lock_blocked",
            "asset/tests/project/asset_flow_sample.rs",
            "asset/tests/project/asset_flow_sample/end_to_end.rs",
            "runtime_15_asset_project_flow_sample_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset project example vampire test folder split",
        &[
            "runtime_15_asset_project_example_vampire_tests_folder_split_static_passed_cargo_deferred",
            "asset/tests/project/example_vampire.rs",
            "asset/tests/project/example_vampire/manifest_scene_imports.rs",
            "asset/tests/project/example_vampire/third_person_render_extract.rs",
            "runtime_15_asset_project_example_vampire_tests_are_folder_backed",
        ],
    ),
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
    (
        "Runtime 15 M3 asset UI test folder split",
        &[
            "runtime_15_asset_ui_tests_folder_split_static_passed_cargo_deferred",
            "asset/tests/assets/ui.rs",
            "asset/tests/assets/ui/importer.rs",
            "asset/tests/assets/ui/project_manager.rs",
            "runtime_15_asset_ui_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 asset pipeline manager test folder split",
        &[
            "runtime_15_asset_pipeline_manager_tests_folder_split_static_passed_cargo_deferred",
            "asset/tests/pipeline/manager.rs",
            "asset/tests/pipeline/manager/model_import.rs",
            "asset/tests/pipeline/manager/watcher.rs",
            "runtime_15_asset_pipeline_manager_tests_are_folder_backed",
        ],
    ),
];

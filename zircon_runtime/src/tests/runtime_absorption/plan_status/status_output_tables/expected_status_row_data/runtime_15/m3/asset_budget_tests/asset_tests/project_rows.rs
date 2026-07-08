type Slice = super::Slice;

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
];

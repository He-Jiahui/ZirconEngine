type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
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

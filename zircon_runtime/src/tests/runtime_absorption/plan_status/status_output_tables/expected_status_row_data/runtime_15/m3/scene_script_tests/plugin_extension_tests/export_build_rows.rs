type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 export build plan test folder split",
        &[
            "runtime_15_export_build_plan_tests_folder_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/export_build_plan.rs",
            "tests/plugin_extensions/export_build_plan/catalog_projection.rs",
            "runtime_15_export_build_plan_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 export build plan profile feature matrix test child-owner split",
        &[
            "runtime_15_export_build_plan_profile_feature_matrix_tests_child_owner_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/export_build_plan.rs",
            "tests/plugin_extensions/export_build_plan/profile_feature_matrix.rs",
            "runtime_15_export_build_plan_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 export build plan platform test folder split",
        &[
            "runtime_15_export_build_plan_platform_tests_folder_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/export_build_plan_platform.rs",
            "tests/plugin_extensions/export_build_plan_platform/browser_hosts.rs",
            "runtime_15_export_build_plan_platform_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 export build plan platform release-adapter test child-owner split",
        &[
            "runtime_15_export_build_plan_platform_release_adapter_tests_child_owner_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/export_build_plan_platform.rs",
            "tests/plugin_extensions/export_build_plan_platform/release_adapters.rs",
            "runtime_15_export_build_plan_platform_tests_are_folder_backed",
        ],
    ),
];

type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 native live-host tests folder split",
        &[
            "runtime_15_native_live_host_tests_folder_split_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/native_plugin_live_host/tests.rs",
            "plugin/native_plugin_loader/native_plugin_live_host/tests/bridge_bindings.rs",
            "plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs",
            "runtime_15_native_live_host_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 native plugin loader real fixture test folder split",
        &[
            "runtime_15_native_plugin_loader_real_fixture_tests_folder_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/native_plugin_loader.rs",
            "tests/plugin_extensions/native_plugin_loader/real_fixture.rs",
            "runtime_15_native_plugin_loader_real_fixture_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 extension registry bridge test folder split",
        &[
            "runtime_15_extension_registry_bridge_tests_folder_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/extension_registry_bridge.rs",
            "tests/plugin_extensions/extension_registry_bridge/basics.rs",
            "tests/plugin_extensions/extension_registry_bridge/diagnostics.rs",
            "runtime_15_extension_registry_bridge_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 manifest contributions test folder split",
        &[
            "runtime_15_manifest_contributions_tests_folder_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/manifest_contributions.rs",
            "tests/plugin_extensions/manifest_contributions/editor_only.rs",
            "tests/plugin_extensions/manifest_contributions/net.rs",
            "runtime_15_manifest_contributions_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 manifest contributions runtime-family test child-owner split",
        &[
            "runtime_15_manifest_contributions_runtime_family_tests_child_owner_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/manifest_contributions.rs",
            "tests/plugin_extensions/manifest_contributions/runtime_family.rs",
            "runtime_15_manifest_contributions_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 runtime plugin package manifest test folder split",
        &[
            "runtime_15_runtime_plugin_package_manifest_tests_folder_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/runtime_plugin_package_manifest.rs",
            "tests/plugin_extensions/runtime_plugin_package_manifest/feature_modules.rs",
            "runtime_15_runtime_plugin_package_manifest_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 runtime plugin package manifest capability-status test child-owner split",
        &[
            "runtime_15_runtime_plugin_package_manifest_capability_status_tests_child_owner_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/runtime_plugin_package_manifest.rs",
            "tests/plugin_extensions/runtime_plugin_package_manifest/capability_status.rs",
            "runtime_15_runtime_plugin_package_manifest_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 runtime plugin catalog feature-dependency report test child-owner split",
        &[
            "runtime_15_runtime_plugin_catalog_features_dependency_report_tests_child_owner_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/runtime_plugin_catalog_features.rs",
            "tests/plugin_extensions/runtime_plugin_catalog_features/feature_dependency_reports.rs",
            "runtime_15_runtime_plugin_catalog_features_dependency_report_tests_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 runtime plugin lifecycle fixture child-owner split",
        &[
            "runtime_15_runtime_plugin_lifecycle_fixture_child_owner_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/runtime_plugin_lifecycle.rs",
            "tests/plugin_extensions/runtime_plugin_lifecycle/lifecycle_fixtures.rs",
            "runtime_15_runtime_plugin_lifecycle_fixture_owner_is_folder_backed",
        ],
    ),
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

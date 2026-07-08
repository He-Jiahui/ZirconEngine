type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
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
];

use super::*;

#[test]
fn runtime_15_runtime_plugin_package_manifest_tests_are_folder_backed() {
    let parent = read_runtime_src("tests/plugin_extensions/runtime_plugin_package_manifest.rs");
    let feature_modules = read_runtime_src(
        "tests/plugin_extensions/runtime_plugin_package_manifest/feature_modules.rs",
    );
    let capability_status = read_runtime_src(
        "tests/plugin_extensions/runtime_plugin_package_manifest/capability_status.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let package_manifest_doc = read_repo("docs/zircon_runtime/plugin/package_manifest.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs",
    );

    assert_contains_all(
        "runtime plugin package manifest parent mounts feature/module child owner",
        &parent,
        &[
            "#[path = \"runtime_plugin_package_manifest/feature_modules.rs\"]",
            "mod feature_modules;",
            "#[path = \"runtime_plugin_package_manifest/capability_status.rs\"]",
            "mod capability_status;",
        ],
    );

    for moved_test in [
        "fn native_runtime_plugin_registration_report_rejects_invalid_package_optional_features",
        "fn native_registration_rejects_duplicate_package_feature_extension_providers",
        "fn native_runtime_plugin_registration_report_rejects_invalid_package_module_identities",
        "fn native_runtime_plugin_registration_report_rejects_duplicate_package_module_names",
        "fn native_runtime_plugin_registration_report_rejects_invalid_capability_status_capabilities",
        "fn native_runtime_plugin_registration_report_rejects_duplicate_capability_status_targets",
        "fn native_runtime_plugin_registration_report_rejects_invalid_capability_status_bevy_metadata",
        "fn valid_sound_timeline_feature",
    ] {
        assert!(
            !parent.contains(moved_test),
            "runtime plugin package manifest parent should mount the feature/module child owner instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "feature/modules child owns optional feature and module validation contracts",
        &feature_modules,
        &[
            "use super::*;",
            "fn native_runtime_plugin_registration_report_rejects_invalid_package_optional_features",
            "fn native_registration_rejects_duplicate_package_feature_extension_providers",
            "fn native_runtime_plugin_registration_report_rejects_invalid_package_module_identities",
            "fn native_runtime_plugin_registration_report_rejects_duplicate_package_module_names",
            "fn valid_sound_timeline_feature",
        ],
    );

    assert_contains_all(
        "capability-status child owns package readiness validation contracts",
        &capability_status,
        &[
            "use super::*;",
            "fn native_runtime_plugin_registration_report_rejects_invalid_capability_status_capabilities",
            "fn native_runtime_plugin_registration_report_rejects_duplicate_capability_status_targets",
            "fn native_runtime_plugin_registration_report_rejects_invalid_capability_status_bevy_metadata",
        ],
    );

    assert_eq!(
        parent.matches("#[test]").count()
            + feature_modules.matches("#[test]").count()
            + capability_status.matches("#[test]").count(),
        36,
        "runtime plugin package manifest parent plus split children should preserve the current 36 tests"
    );

    for (path, source) in [
        (
            "tests/plugin_extensions/runtime_plugin_package_manifest.rs",
            parent.as_str(),
        ),
        (
            "tests/plugin_extensions/runtime_plugin_package_manifest/feature_modules.rs",
            feature_modules.as_str(),
        ),
        (
            "tests/plugin_extensions/runtime_plugin_package_manifest/capability_status.rs",
            capability_status.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("package manifest doc", package_manifest_doc.as_str()),
        ("status-output scene/script row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 runtime plugin package manifest test folder split",
                "runtime_15_runtime_plugin_package_manifest_tests_folder_split_static_passed_cargo_deferred",
                "Runtime 15 M3 runtime plugin package manifest capability-status test child-owner split",
                "runtime_15_runtime_plugin_package_manifest_capability_status_tests_child_owner_split_static_passed_cargo_deferred",
                "tests/plugin_extensions/runtime_plugin_package_manifest.rs",
                "tests/plugin_extensions/runtime_plugin_package_manifest/feature_modules.rs",
                "tests/plugin_extensions/runtime_plugin_package_manifest/capability_status.rs",
                "runtime_15_runtime_plugin_package_manifest_tests_are_folder_backed",
            ],
        );
    }
}

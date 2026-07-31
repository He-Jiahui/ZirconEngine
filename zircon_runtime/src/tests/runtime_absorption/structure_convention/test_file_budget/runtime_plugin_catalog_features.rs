use super::*;

#[test]
fn runtime_15_runtime_plugin_catalog_features_dependency_report_tests_are_child_owner() {
    let parent = read_runtime_src("tests/plugin_extensions/runtime_plugin_catalog_features.rs");
    let feature_dependency_reports = read_runtime_src(
        "tests/plugin_extensions/runtime_plugin_catalog_features/feature_dependency_reports.rs",
    );
    let runtime_15_plan =
        read_repo(
            "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
        );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let optional_feature_doc =
        read_repo("docs/engine-architecture/plugin-optional-feature-bundles.md");
    let package_manifest_doc = read_repo("docs/zircon_runtime/plugin/package_manifest.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs",
    );

    assert_contains_all(
        "runtime plugin catalog features parent mounts feature dependency report child owner",
        &parent,
        &[
            "#[path = \"runtime_plugin_catalog_features/feature_dependency_reports.rs\"]",
            "mod feature_dependency_reports;",
        ],
    );

    for moved_test in [
        "fn runtime_plugin_catalog_reports_optional_feature_dependency_status",
        "fn runtime_plugin_catalog_gates_external_feature_packages_on_provider_selection",
        "fn runtime_plugin_catalog_rejects_secondary_primary_feature_dependency",
        "fn runtime_plugin_catalog_reports_target_mismatch_for_optional_feature",
        "fn earlier_provider_capability_is_visible_to_later_immediate_blocker",
        "fn later_provider_does_not_rewrite_an_immediate_blocker",
        "fn immediate_blocker_is_not_an_unresolved_cycle_provider",
        "fn runtime_plugin_catalog_reports_feature_capability_cycles",
        "fn runtime_plugin_catalog_reports_disabled_feature_provider_as_missing_capability",
        "fn runtime_plugin_catalog_reports_self_feature_capability_cycle",
    ] {
        assert!(
            !parent.contains(moved_test),
            "runtime plugin catalog feature parent should mount the child owner instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "feature dependency report child owns optional feature dependency diagnostics",
        &feature_dependency_reports,
        &[
            "use super::*;",
            "fn runtime_plugin_catalog_reports_optional_feature_dependency_status",
            "fn runtime_plugin_catalog_gates_external_feature_packages_on_provider_selection",
            "fn runtime_plugin_catalog_rejects_secondary_primary_feature_dependency",
            "fn runtime_plugin_catalog_reports_target_mismatch_for_optional_feature",
            "fn earlier_provider_capability_is_visible_to_later_immediate_blocker",
            "fn later_provider_does_not_rewrite_an_immediate_blocker",
            "fn immediate_blocker_is_not_an_unresolved_cycle_provider",
            "fn runtime_plugin_catalog_reports_feature_capability_cycles",
            "fn runtime_plugin_catalog_reports_disabled_feature_provider_as_missing_capability",
            "fn runtime_plugin_catalog_reports_self_feature_capability_cycle",
            "feature_dependency_report",
            "feature capability dependencies form a cycle",
        ],
    );

    assert_eq!(
        parent.matches("#[test]").count() + feature_dependency_reports.matches("#[test]").count(),
        20,
        "runtime plugin catalog feature parent plus dependency-report child should preserve the current 20 tests"
    );

    for (path, source) in [
        (
            "tests/plugin_extensions/runtime_plugin_catalog_features.rs",
            parent.as_str(),
        ),
        (
            "tests/plugin_extensions/runtime_plugin_catalog_features/feature_dependency_reports.rs",
            feature_dependency_reports.as_str(),
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
        ("optional feature bundle doc", optional_feature_doc.as_str()),
        ("package manifest doc", package_manifest_doc.as_str()),
        ("status-output scene/script row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 runtime plugin catalog feature-dependency report test child-owner split",
                "runtime_15_runtime_plugin_catalog_features_dependency_report_tests_child_owner_split_static_passed_cargo_deferred",
                "tests/plugin_extensions/runtime_plugin_catalog_features.rs",
                "tests/plugin_extensions/runtime_plugin_catalog_features/feature_dependency_reports.rs",
                "runtime_15_runtime_plugin_catalog_features_dependency_report_tests_are_child_owner",
            ],
        );
    }
}

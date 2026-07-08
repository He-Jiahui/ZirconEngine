use super::*;

#[test]
fn runtime_15_runtime_plugin_lifecycle_fixture_owner_is_folder_backed() {
    let parent = read_runtime_src("tests/plugin_extensions/runtime_plugin_lifecycle.rs");
    let fixtures =
        read_runtime_src("tests/plugin_extensions/runtime_plugin_lifecycle/lifecycle_fixtures.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = [
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/plugin_extension_tests.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/plugin_extension_tests/runtime_catalog_rows.rs",
        ),
    ]
    .join("\n");

    assert_contains_all(
        "runtime plugin lifecycle parent mounts lifecycle fixture child owner",
        &parent,
        &[
            "#[path = \"runtime_plugin_lifecycle/lifecycle_fixtures.rs\"]",
            "mod lifecycle_fixtures;",
            "use lifecycle_fixtures::*;",
        ],
    );

    for moved_fixture in [
        "struct OptionalDependencyProbe",
        "struct LifecycleOrderPlugin",
        "struct ReadyOrderPlugin",
        "struct OrderedLifecyclePlugin",
        "struct LifecycleOrderFeature",
        "struct ReadyOrderFeature",
    ] {
        assert!(
            !parent.contains(moved_fixture),
            "runtime plugin lifecycle parent should mount the fixture child instead of defining {moved_fixture}"
        );
    }

    assert_contains_all(
        "runtime plugin lifecycle fixture child owns lifecycle helper types",
        &fixtures,
        &[
            "use super::*;",
            "pub(super) struct OptionalDependencyProbe",
            "pub(super) struct LifecycleOrderPlugin",
            "pub(super) struct ReadyOrderPlugin",
            "pub(super) struct OrderedLifecyclePlugin",
            "pub(super) struct LifecycleOrderFeature",
            "pub(super) struct ReadyOrderFeature",
            "impl RuntimePlugin for OptionalDependencyProbe",
            "impl RuntimePluginFeature for ReadyOrderFeature",
        ],
    );

    assert_eq!(
        parent.matches("#[test]").count() + fixtures.matches("#[test]").count(),
        11,
        "runtime plugin lifecycle parent plus fixture child should preserve the current 11 tests"
    );
    assert_eq!(
        fixtures.matches("#[test]").count(),
        0,
        "runtime plugin lifecycle fixture child should only own helper types"
    );

    for (path, source) in [
        (
            "tests/plugin_extensions/runtime_plugin_lifecycle.rs",
            parent.as_str(),
        ),
        (
            "tests/plugin_extensions/runtime_plugin_lifecycle/lifecycle_fixtures.rs",
            fixtures.as_str(),
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
        (
            "status-output plugin extension row data",
            status_rows.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 runtime plugin lifecycle fixture child-owner split",
                "runtime_15_runtime_plugin_lifecycle_fixture_child_owner_split_static_passed_cargo_deferred",
                "tests/plugin_extensions/runtime_plugin_lifecycle.rs",
                "tests/plugin_extensions/runtime_plugin_lifecycle/lifecycle_fixtures.rs",
                "runtime_15_runtime_plugin_lifecycle_fixture_owner_is_folder_backed",
            ],
        );
    }
}

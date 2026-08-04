use super::*;

#[test]
fn runtime_15_extension_registry_bridge_tests_are_folder_backed() {
    let parent = read_runtime_src("tests/plugin_extensions/extension_registry_bridge.rs");
    let basics = read_runtime_src("tests/plugin_extensions/extension_registry_bridge/basics.rs");
    let diagnostics =
        read_runtime_src("tests/plugin_extensions/extension_registry_bridge/diagnostics.rs");
    let lifecycle =
        read_runtime_src("tests/plugin_extensions/extension_registry_bridge/lifecycle.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let plugin_bridge_doc = read_repo("docs/zircon_runtime/plugin/bridge.md");

    assert_contains_all(
        "extension registry bridge parent mounts child owners",
        &parent,
        &[
            "#[path = \"extension_registry_bridge/basics.rs\"]",
            "mod basics;",
            "#[path = \"extension_registry_bridge/diagnostics.rs\"]",
            "mod diagnostics;",
            "#[path = \"extension_registry_bridge/lifecycle.rs\"]",
            "mod lifecycle;",
        ],
    );

    for moved_test in [
        "fn duplicate_interface_export_rejected",
        "fn bridge_table_summarizes_diagnostics_for_matrix",
        "fn bridge_table_reports_owner_enabled_transition",
        "fn pin_guard_amortizes_weak_bridge_resolution",
    ] {
        assert!(
            !parent.contains(moved_test),
            "extension registry bridge parent should mount child owners instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "bridge basics child owns core bridge behavior",
        &basics,
        &[
            "use super::*;",
            "fn duplicate_interface_export_rejected",
            "fn generation_parity_encodes_enabled_state",
            "fn pin_guard_amortizes_weak_bridge_resolution",
            "struct CountingWeatherProvider",
        ],
    );
    assert_contains_all(
        "bridge diagnostics child owns matrix and snapshot behavior",
        &diagnostics,
        &[
            "use super::*;",
            "fn bridge_table_reports_interface_status_for_diagnostics",
            "fn bridge_table_summarizes_diagnostics_for_matrix",
            "fn bridge_diagnostics_matrix_projects_editor_rows",
        ],
    );
    assert_contains_all(
        "bridge lifecycle child owns owner transition behavior",
        &lifecycle,
        &[
            "use super::*;",
            "fn hot_reload_swaps_provider_without_caller_rewiring",
            "fn bridge_table_reloads_owner_exports_with_report",
            "fn bridge_table_reports_owner_deactivation_transition",
        ],
    );

    let moved_test_count = [basics.as_str(), diagnostics.as_str(), lifecycle.as_str()]
        .iter()
        .map(|source| source.matches("#[test]").count())
        .sum::<usize>();
    assert_eq!(
        parent.matches("#[test]").count() + moved_test_count,
        20,
        "extension registry bridge parent plus split children should preserve the original 20 tests"
    );

    for (path, source) in [
        (
            "tests/plugin_extensions/extension_registry_bridge.rs",
            parent.as_str(),
        ),
        (
            "tests/plugin_extensions/extension_registry_bridge/basics.rs",
            basics.as_str(),
        ),
        (
            "tests/plugin_extensions/extension_registry_bridge/diagnostics.rs",
            diagnostics.as_str(),
        ),
        (
            "tests/plugin_extensions/extension_registry_bridge/lifecycle.rs",
            lifecycle.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

use super::*;

#[test]
fn runtime_15_native_live_host_tests_are_folder_backed() {
    let parent = read_runtime_src("plugin/native_plugin_loader/native_plugin_live_host/tests.rs");
    let runtime_behavior = read_runtime_src(
        "plugin/native_plugin_loader/native_plugin_live_host/tests/runtime_behavior.rs",
    );
    let bridge_bindings = read_runtime_src(
        "plugin/native_plugin_loader/native_plugin_live_host/tests/bridge_bindings.rs",
    );
    let hot_reload_state = read_runtime_src(
        "plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let plugin_bridge_doc = read_repo("docs/zircon_runtime/plugin/bridge.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs",
    );

    assert_contains_all(
        "native live-host parent test module mounts child owners",
        &parent,
        &[
            "#[path = \"tests/bridge_bindings.rs\"]",
            "mod bridge_bindings;",
            "#[path = \"tests/hot_reload_state.rs\"]",
            "mod hot_reload_state;",
            "#[path = \"tests/runtime_behavior.rs\"]",
            "mod runtime_behavior;",
        ],
    );

    for moved_test in [
        "fn native_live_host_runtime_descriptor_includes_validation_report",
        "fn native_live_host_reuses_installed_bridge_bindings_for_loaded_manifest_scopes",
        "fn native_hot_reload_state_saves_and_restores_runtime_snapshot",
        "fn hot_reload_failure_rolls_back_to_snapshot",
    ] {
        assert!(
            !parent.contains(moved_test),
            "native live-host parent should mount child owners instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "runtime behavior child owns runtime descriptor and snapshot contracts",
        &runtime_behavior,
        &[
            "use super::*;",
            "fn native_live_host_runtime_descriptor_includes_validation_report",
            "fn native_live_host_runtime_broadcasts_and_snapshots_empty_when_no_plugins_loaded",
            "fn native_live_host_runtime_snapshot_restore_skips_schema_mismatch",
        ],
    );
    assert_contains_all(
        "bridge bindings child owns native bridge call scope contracts",
        &bridge_bindings,
        &[
            "use super::*;",
            "fn native_live_host_builds_bridge_call_scope_from_loaded_manifest",
            "fn native_live_host_reuses_installed_bridge_bindings_for_loaded_manifest_scopes",
            "fn native_live_host_reloads_bridge_lifecycle_and_installed_binding_scope",
            "fn native_live_host_rejects_loaded_manifest_bridge_method_without_binding",
        ],
    );
    assert_contains_all(
        "hot reload state child owns runtime snapshot rollback contracts",
        &hot_reload_state,
        &[
            "use super::*;",
            "fn native_live_host_rollback_plan_restores_existing_plugin_when_reload_fails_before_unload",
            "fn native_hot_reload_state_saves_and_restores_runtime_snapshot",
            "fn hot_reload_failure_rolls_back_to_snapshot",
        ],
    );

    let moved_test_count = [
        runtime_behavior.as_str(),
        bridge_bindings.as_str(),
        hot_reload_state.as_str(),
    ]
    .iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        parent.matches("#[test]").count() + moved_test_count,
        27,
        "native live-host parent plus split children should preserve the original 27 tests"
    );

    for (path, source) in [
        (
            "plugin/native_plugin_loader/native_plugin_live_host/tests.rs",
            parent.as_str(),
        ),
        (
            "plugin/native_plugin_loader/native_plugin_live_host/tests/runtime_behavior.rs",
            runtime_behavior.as_str(),
        ),
        (
            "plugin/native_plugin_loader/native_plugin_live_host/tests/bridge_bindings.rs",
            bridge_bindings.as_str(),
        ),
        (
            "plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs",
            hot_reload_state.as_str(),
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
        ("plugin bridge doc", plugin_bridge_doc.as_str()),
        ("status-output scene/script row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 native live-host tests folder split",
                "runtime_15_native_live_host_tests_folder_split_static_passed_cargo_deferred",
                "plugin/native_plugin_loader/native_plugin_live_host/tests.rs",
                "plugin/native_plugin_loader/native_plugin_live_host/tests/bridge_bindings.rs",
                "plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs",
                "runtime_15_native_live_host_tests_are_folder_backed",
            ],
        );
    }
}

use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_native_host_api_adapter_tests_are_child_owner() {
    let parent = read_runtime_src("plugin/native_plugin_loader/host_api_adapter.rs");
    let tests = read_runtime_src("plugin/native_plugin_loader/host_api_adapter/tests.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let plugin_bridge_doc = read_repo("docs/zircon_runtime/plugin/bridge.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m4_surface_cleanup.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m4_surface_cleanup.rs",
    );

    assert_contains_all(
        "native host API adapter parent keeps ABI entrypoints and mounts tests child",
        &parent,
        &[
            "#[path = \"host_api_adapter/tests.rs\"]",
            "mod tests;",
            "unsafe extern \"C\" fn native_host_register_system_v1",
            "unsafe extern \"C\" fn native_host_bridge_call_v1",
            "unsafe fn native_host_bridge_call_v1_inner",
            "fn status(code: ZrStatusCode) -> ZrStatus",
        ],
    );
    for moved_test in [
        "fn native_host_api_v3_registers_systems_and_components_into_runtime_registry",
        "fn native_host_bridge_call_scope_dispatches_registered_method",
        "fn native_bridge_method_descriptors_use_package_manifest_metadata",
        "fn native_host_api_v3_preserves_dotted_plugin_ids",
    ] {
        assert!(
            !parent.contains(moved_test),
            "host_api_adapter.rs should delegate {moved_test} to host_api_adapter/tests.rs"
        );
    }
    assert_contains_all(
        "native host API adapter tests child owns registration and bridge coverage",
        &tests,
        &[
            "fn native_host_api_v3_registers_systems_and_components_into_runtime_registry",
            "fn native_host_bridge_call_scope_dispatches_registered_method",
            "NativeHostBridgeCallScope::from_method_descriptors",
            "NativeBridgeMethodManifestError::MissingBinding",
            "PluginPackageManifest::new(\"weather\", \"Weather\")",
            "fn native_host_api_v3_preserves_dotted_plugin_ids",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count() + tests.matches("#[test]").count(),
        15,
        "native host API adapter parent plus split child should preserve the current 15 tests"
    );
    for (path, source) in [
        (
            "plugin/native_plugin_loader/host_api_adapter.rs",
            parent.as_str(),
        ),
        (
            "plugin/native_plugin_loader/host_api_adapter/tests.rs",
            tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production/test owner budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("plugin bridge doc", plugin_bridge_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 native host API adapter tests owner split",
                "runtime_15_native_host_api_adapter_tests_owner_split_static_passed_cargo_deferred",
                "plugin/native_plugin_loader/host_api_adapter.rs",
                "plugin/native_plugin_loader/host_api_adapter/tests.rs",
                "runtime_15_native_host_api_adapter_tests_are_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output status map",
        &status_map,
        &[
            "Runtime 15 M4 native host API adapter tests owner split",
            "runtime_15_native_host_api_adapter_tests_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "status-output date map",
        &date_map,
        &[
            "Runtime 15 M4 native host API adapter tests owner split",
            "2026-06-24",
        ],
    );
}

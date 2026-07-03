#[test]
fn review_f5_native_live_host_bridge_lifecycle_uses_typed_error() {
    let bridge_lifecycle = include_str!(
        "../../../../../plugin/native_plugin_loader/native_plugin_live_host/bridge_lifecycle.rs"
    );
    let live_host_root =
        include_str!("../../../../../plugin/native_plugin_loader/native_plugin_live_host.rs");
    let live_host_tests =
        include_str!("../../../../../plugin/native_plugin_loader/native_plugin_live_host/tests.rs");
    let native_boundary =
        include_str!("../../../../../../../docs/engine-architecture/native-plugin-boundary.md");
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let module_convention =
        include_str!("../../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let status_rows = include_str!(
        "../../../plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs"
    );
    let status_map = include_str!(
        "../../../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs"
    );
    let date_map = include_str!(
        "../../../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs"
    );

    for required in [
        "type NativePluginBridgeLifecycleResult<T>",
        "std::result::Result<T, NativePluginBridgeLifecycleError>",
        "enum NativePluginBridgeLifecycleError",
        "Load",
        "HotReload",
        "Unload",
        "BridgeLifecycleRejected",
        "UnloadRollback",
        "impl std::fmt::Display for NativePluginBridgeLifecycleError",
        "impl std::error::Error for NativePluginBridgeLifecycleError",
        ") -> NativePluginBridgeLifecycleResult<NativePluginRuntimeHotUpdateReport>",
        ") -> NativePluginBridgeLifecycleResult<NativePluginLiveHostLoadReport>",
        ") -> NativePluginBridgeLifecycleResult<NativePluginLiveHostOutcome>",
        "NativePluginBridgeLifecycleError::BridgeLifecycleRejected",
        "NativePluginBridgeLifecycleError::UnloadRollback",
    ] {
        assert!(
            bridge_lifecycle.contains(required),
            "native live-host bridge-lifecycle typed-error owner should contain `{required}`"
        );
    }

    let production = bridge_lifecycle
        .split("#[cfg(test)]")
        .next()
        .expect("native live-host bridge-lifecycle production source");
    for forbidden in [
        "return Err(bridge_report.diagnostic());",
        "Err(format!(\"{error}; {}\", rollback_report.diagnostic()))",
        "let mut report = self.hot_reload_runtime_plugins_from_export_root(export_root)?;",
        "let mut report = self.load_runtime_plugins_from_export_root(export_root)?;",
        "let mut outcome = self.hot_reload_runtime_plugin(root, plugin_id)?;",
    ] {
        assert!(
            !production.contains(forbidden),
            "native live-host bridge-lifecycle owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    assert!(
        bridge_lifecycle.contains(".map_err(|error| error.to_string())"),
        "native live-host bridge-lifecycle public APIs should stringify typed errors only at public boundaries"
    );
    assert!(
        live_host_root.contains("NativePluginBridgeLifecycleError"),
        "native live-host test root should expose the typed bridge-lifecycle error to module-local tests"
    );

    for required_test in [
        "native_live_host_bridge_lifecycle_rejected_unload_reports_typed_error",
        "NativePluginBridgeLifecycleError::BridgeLifecycleRejected",
    ] {
        assert!(
            live_host_tests.contains(required_test),
            "native live-host bridge-lifecycle tests should contain `{required_test}`"
        );
    }

    for doc_anchor in [
        "Runtime 15 F5 native live-host bridge lifecycle typed errors",
        "runtime_15_native_live_host_bridge_lifecycle_typed_errors_static_passed_cargo_deferred",
        "review_f5_native_live_host_bridge_lifecycle_uses_typed_error",
        "plugin/native_plugin_loader/native_plugin_live_host/bridge_lifecycle.rs",
        "NativePluginBridgeLifecycleError::BridgeLifecycleRejected",
        "native live-host bridge lifecycle keeps string diagnostics at public live-host boundaries",
    ] {
        assert!(
            native_boundary.contains(doc_anchor)
                || review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || module_convention.contains(doc_anchor)
                || status_rows.contains(doc_anchor)
                || status_map.contains(doc_anchor)
                || date_map.contains(doc_anchor),
            "native live-host bridge-lifecycle docs/status should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f5_native_live_host_bridge_methods_use_typed_error() {
    let bridge_methods = include_str!(
        "../../../../../../../plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs"
    );
    let live_host_root =
        include_str!("../../../../../../../plugin/native_plugin_loader/native_plugin_live_host.rs");
    let bridge_method_tests = include_str!(
        "../../../../../../../plugin/native_plugin_loader/native_plugin_live_host/tests/bridge_bindings.rs"
    );
    let native_boundary = include_str!(
        "../../../../../../../../../docs/engine-architecture/native-plugin-boundary.md"
    );
    let review_findings = include_str!(
        "../../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
    );
    let runtime_15_plan = include_str!(
        "../../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
    );
    let runtime_index =
        include_str!("../../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md");
    let convention =
        include_str!("../../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md");
    let module_convention = include_str!(
        "../../../../../../../../../docs/zircon_runtime/structure/module-convention.md"
    );
    let status_rows = include_str!(
        "../../../../../plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs"
    );
    let status_map = include_str!(
        "../../../../../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs"
    );
    let date_map = include_str!(
        "../../../../../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs"
    );

    for required in [
        "type NativePluginBridgeMethodResult<T>",
        "std::result::Result<T, NativePluginBridgeMethodError>",
        "enum NativePluginBridgeMethodError",
        "RuntimePluginNotLoaded",
        "MissingDiscoveredBridgeMethodTable",
        "MissingPackageManifest",
        "MissingInstalledBridgeMethodBindings",
        "InvalidBridgeMethodManifest",
        "BridgeCallScope",
        "BridgeLifecycleRejected",
        "MissingDeclaredBridgeMethod",
        "impl std::fmt::Display for NativePluginBridgeMethodError",
        "impl std::error::Error for NativePluginBridgeMethodError",
        ") -> NativePluginBridgeMethodResult<usize>",
        ") -> NativePluginBridgeMethodResult<()>",
        ") -> NativePluginBridgeMethodResult<NativeHostBridgeCallScope>",
        ") -> NativePluginBridgeMethodResult<NativePluginLiveHostBridgeReloadReport>",
        ") -> NativePluginBridgeMethodResult<u32>",
        "NativePluginBridgeMethodError::MissingPackageManifest",
        "NativePluginBridgeMethodError::MissingDeclaredBridgeMethod",
        ".map_err(|error| error.to_string())",
    ] {
        assert!(
            bridge_methods.contains(required),
            "native live-host bridge methods typed-error owner should contain `{required}`"
        );
    }

    let production = bridge_methods
        .split("#[cfg(test)]")
        .next()
        .expect("native live-host bridge methods production source");
    for forbidden in [
        "use super::diagnostics::unloaded_plugin_error;",
        "format!(\"runtime plugin {plugin_id} exposes no native bridge method table\")",
        "format!(\"runtime plugin {plugin_id} has no package manifest\")",
        "format!(\"runtime plugin {plugin_id} has no installed native bridge method bindings\")",
        "format!(\n            \"runtime plugin {plugin_id} package manifest does not declare bridge method",
        "format!(\n            \"runtime plugin {} has no package manifest",
        ".map_err(|error| error.to_string())?",
    ] {
        assert!(
            !production.contains(forbidden),
            "native live-host bridge methods owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    assert!(
        live_host_root.contains("NativePluginBridgeMethodError"),
        "native live-host test root should expose the typed bridge method error to module-local tests"
    );

    for required_test in [
        "native_live_host_bridge_methods_report_typed_missing_manifest_error",
        "native_live_host_bridge_methods_report_typed_missing_method_slot_error",
        "NativePluginBridgeMethodError::MissingPackageManifest",
        "NativePluginBridgeMethodError::MissingDeclaredBridgeMethod",
    ] {
        assert!(
            bridge_method_tests.contains(required_test),
            "native live-host bridge method tests should contain `{required_test}`"
        );
    }

    for doc_anchor in [
        "Runtime 15 F5 native live-host bridge methods typed errors",
        "runtime_15_native_live_host_bridge_methods_typed_errors_static_passed_cargo_deferred",
        "review_f5_native_live_host_bridge_methods_use_typed_error",
        "plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs",
        "NativePluginBridgeMethodError::MissingDeclaredBridgeMethod",
        "native live-host bridge methods keep string diagnostics at public live-host boundaries",
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
            "native live-host bridge method docs/status should record `{doc_anchor}`"
        );
    }
}

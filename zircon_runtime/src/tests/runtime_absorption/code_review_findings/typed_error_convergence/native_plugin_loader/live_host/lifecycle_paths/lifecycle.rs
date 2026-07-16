#[test]
fn review_f5_native_live_host_lifecycle_uses_typed_error() {
    let lifecycle = include_str!(
        "../../../../../../../plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs"
    );
    let bridge_methods = include_str!(
        "../../../../../../../plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs"
    );
    let hot_update_application = include_str!(
        "../../../../../../../plugin/native_plugin_loader/native_plugin_live_host/hot_update_application.rs"
    );
    let live_host_root =
        include_str!("../../../../../../../plugin/native_plugin_loader/native_plugin_live_host.rs");
    let live_host_tests = include_str!(
        "../../../../../../../plugin/native_plugin_loader/native_plugin_live_host/tests.rs"
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
        "type NativePluginLiveHostLifecycleResult<T>",
        "std::result::Result<T, NativePluginLiveHostLifecycleError>",
        "enum NativePluginLiveHostLifecycleError",
        "LiveHostLock",
        "RuntimePluginNotLoaded",
        "UnloadBehavior",
        "HotReloadDidNotLoad",
        "HotReloadSnapshot",
        "HotReloadUnloadBeforeReload",
        "HotReloadRestore",
        "RuntimeBridgeMethodBindings",
        "UnsupportedLiveHostModuleKind",
        "impl std::fmt::Display for NativePluginLiveHostLifecycleError",
        "impl std::error::Error for NativePluginLiveHostLifecycleError",
        ") -> NativePluginLiveHostLifecycleResult<NativePluginLiveHostOutcome>",
        ") -> NativePluginLiveHostLifecycleResult<NativePluginLoadReport>",
        "NativePluginLiveHostLifecycleError::RuntimePluginNotLoaded",
        "NativePluginLiveHostLifecycleError::HotReloadDidNotLoad",
        "NativePluginLiveHostLifecycleError::HotReloadSnapshot",
        "NativePluginLiveHostLifecycleError::HotReloadRestore",
        "NativePluginLiveHostLifecycleError::RuntimeBridgeMethodBindings",
    ] {
        assert!(
            lifecycle.contains(required),
            "native live-host lifecycle typed-error owner should contain `{required}`"
        );
    }

    let production = lifecycle
        .split("#[cfg(test)]")
        .next()
        .expect("native live-host lifecycle production source");
    for forbidden in [
        "use super::diagnostics::{\n    diagnostics_for_plugin, diagnostics_from_behavior_report, load_report_diagnostics,\n    unloaded_plugin_error,",
        "return Err(unloaded_plugin_error(plugin_id, module_kind));",
        "let error = format!(\n                \"plugin {plugin_id} hot reload did not load",
        "return Err(error.to_string());",
        "return Err(reload_state.rollback_error(format!(",
        "discovered_runtime_bridge_method_bindings(&plugin)",
        "self.replace_runtime_bridge_method_bindings(plugin_id, bindings)?",
        "PluginModuleKind::Native | PluginModuleKind::Vm => Err(format!(",
    ] {
        assert!(
            !production.contains(forbidden),
            "native live-host lifecycle owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    assert!(
        lifecycle.contains(".map_err(|error| error.to_string())")
            && lifecycle.contains("pub(super) fn hot_reload_reported_plugin(")
            && hot_update_application.contains("Err(error) => diagnostics.push(error)"),
        "native live-host lifecycle should stringify typed lifecycle errors only at public/report boundaries"
    );
    assert!(
        bridge_methods.contains("NativePluginBridgeMethodError"),
        "native live-host lifecycle bridge binding failure should preserve typed bridge-method sources"
    );
    assert!(
        live_host_root.contains("NativePluginLiveHostLifecycleError"),
        "native live-host test root should expose the typed lifecycle error to module-local tests"
    );

    for required_test in [
        "native_live_host_lifecycle_unload_reports_typed_unloaded_error",
        "native_live_host_lifecycle_rejects_unmanaged_module_kind_with_typed_error",
        "NativePluginLiveHostLifecycleError::RuntimePluginNotLoaded",
        "NativePluginLiveHostLifecycleError::UnsupportedLiveHostModuleKind",
    ] {
        assert!(
            live_host_tests.contains(required_test),
            "native live-host lifecycle tests should contain `{required_test}`"
        );
    }

    for doc_anchor in [
        "Runtime 15 F5 native live-host lifecycle typed errors",
        "runtime_15_native_live_host_lifecycle_typed_errors_static_passed_cargo_deferred",
        "review_f5_native_live_host_lifecycle_uses_typed_error",
        "plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs",
        "NativePluginLiveHostLifecycleError::HotReloadDidNotLoad",
        "native live-host lifecycle keeps string diagnostics at public live-host and hot-update report boundaries",
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
            "native live-host lifecycle docs/status should record `{doc_anchor}`"
        );
    }
}

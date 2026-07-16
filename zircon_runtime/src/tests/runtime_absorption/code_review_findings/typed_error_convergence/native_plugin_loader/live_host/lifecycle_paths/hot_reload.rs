#[test]
fn review_f5_native_live_host_hot_reload_uses_typed_error() {
    let hot_reload = include_str!(
        "../../../../../../../plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs"
    );
    let lifecycle = include_str!(
        "../../../../../../../plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs"
    );
    let live_host_root =
        include_str!("../../../../../../../plugin/native_plugin_loader/native_plugin_live_host.rs");
    let hot_reload_tests = include_str!(
        "../../../../../../../plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs"
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
        "type NativePluginHotReloadResult<T>",
        "std::result::Result<T, NativePluginHotReloadError>",
        "enum NativePluginHotReloadError",
        "SaveRuntimeState",
        "MissingRuntimeStatePayload",
        "StateSchemaMismatch",
        "RestoreRuntimeState",
        "impl std::fmt::Display for NativePluginHotReloadError",
        "impl std::error::Error for NativePluginHotReloadError",
        ") -> NativePluginHotReloadResult<Option<&PluginStateSnapshot>>",
        ") -> NativePluginHotReloadResult<Vec<String>>",
        "NativePluginHotReloadError::SaveRuntimeState",
        "NativePluginHotReloadError::MissingRuntimeStatePayload",
        "NativePluginHotReloadError::StateSchemaMismatch",
        "NativePluginHotReloadError::RestoreRuntimeState",
    ] {
        assert!(
            hot_reload.contains(required),
            "native live-host hot reload typed-error owner should contain `{required}`"
        );
    }

    let production = hot_reload
        .split("#[cfg(test)]")
        .next()
        .expect("native live-host hot reload production source");
    for forbidden in [
        ") -> Result<Option<&PluginStateSnapshot>, String>",
        ") -> Result<Vec<String>, String>",
        "return Err(format!(\n                \"plugin {plugin_id} hot reload failed while saving runtime state",
        "return Err(format!(\n                \"plugin {plugin_id} hot reload failed because runtime save-state returned no payload",
        "return Err(format!(\n            \"plugin {} hot reload restore-state skipped because snapshot state schema",
        "return Err(format!(\n            \"plugin {} hot reload failed while restoring runtime state",
    ] {
        assert!(
            !production.contains(forbidden),
            "native live-host hot reload owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    assert!(
        lifecycle.contains("NativePluginLiveHostLifecycleError::HotReloadSnapshot")
            && lifecycle.contains("NativePluginLiveHostLifecycleError::HotReloadRestore")
            && lifecycle.contains("restore_error.to_string()")
            && lifecycle.contains(".map_err(|error| error.to_string())"),
        "native live-host lifecycle should preserve hot-reload typed errors and stringify them only at public/rollback diagnostic boundaries"
    );
    assert!(
        live_host_root.contains("NativePluginHotReloadError"),
        "native live-host test root should expose the typed hot-reload error to module-local tests"
    );

    for required_test in [
        "native_hot_reload_snapshot_save_reports_typed_status_error",
        "NativePluginHotReloadError::SaveRuntimeState",
        "NativePluginHotReloadError::StateSchemaMismatch",
    ] {
        assert!(
            hot_reload_tests.contains(required_test),
            "native live-host hot reload tests should contain `{required_test}`"
        );
    }

    for doc_anchor in [
        "Runtime 15 F5 native live-host hot reload typed errors",
        "runtime_15_native_live_host_hot_reload_typed_errors_static_passed_cargo_deferred",
        "review_f5_native_live_host_hot_reload_uses_typed_error",
        "plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs",
        "NativePluginHotReloadError::StateSchemaMismatch",
        "native live-host hot reload keeps string diagnostics at public lifecycle and rollback boundaries",
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
            "native live-host hot reload docs/status should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f5_native_live_host_loading_uses_typed_error() {
    let loading = include_str!(
        "../../../../../../../plugin/native_plugin_loader/native_plugin_live_host/loading.rs"
    );
    let bridge_methods = include_str!(
        "../../../../../../../plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs"
    );
    let registration_replay = include_str!(
        "../../../../../../../plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs"
    );
    let runtime_behavior = include_str!(
        "../../../../../../../plugin/native_plugin_loader/native_plugin_live_host/runtime_behavior.rs"
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
        "../../../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
    );
    let runtime_15_plan = include_str!(
        "../../../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
    );
    let runtime_index =
        include_str!("../../../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md");
    let convention =
        include_str!("../../../../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md");
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
        "type NativePluginLiveHostLoadingResult<T>",
        "std::result::Result<T, NativePluginLiveHostLoadingError>",
        "enum NativePluginLiveHostLoadingError",
        "LiveHostLockPoisoned",
        "UnloadBeforeReload",
        "RuntimeBridgeMethodBindings",
        "impl std::fmt::Display for NativePluginLiveHostLoadingError",
        "impl std::error::Error for NativePluginLiveHostLoadingError",
        ") -> NativePluginLiveHostLoadingResult<NativePluginLiveHostLoadReport>",
        ") -> NativePluginLiveHostLoadingResult<MutexGuard<'_, BTreeMap<String, LoadedNativePlugin>>>",
        "NativePluginLiveHostLoadingError::LiveHostLockPoisoned",
        "NativePluginLiveHostLoadingError::UnloadBeforeReload",
        "NativePluginLiveHostLoadingError::RuntimeBridgeMethodBindings",
    ] {
        assert!(
            loading.contains(required),
            "native live-host loading typed-error owner should contain `{required}`"
        );
    }

    let production = loading
        .split("#[cfg(test)]")
        .next()
        .expect("native live-host loading production source");
    for forbidden in [
        ") -> Result<MutexGuard<'_, BTreeMap<String, LoadedNativePlugin>>, String>",
        ".map_err(|_| \"native plugin live host lock is poisoned\".to_string())",
        "discovered_runtime_bridge_method_bindings(&plugin)",
        "self.replace_runtime_bridge_method_bindings(&plugin_id, bindings)?",
        "return Err(error);",
    ] {
        assert!(
            !production.contains(forbidden),
            "native live-host loading owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    for required_source in [
        "use super::loading::{lock_loaded_native_plugins, NativePluginLiveHostLoadingError};",
        "LiveHostLock(NativePluginLiveHostLoadingError)",
        "Self::LiveHostLock(error) => Some(error)",
    ] {
        assert!(
            bridge_methods.contains(required_source)
                || registration_replay.contains(required_source)
                || runtime_behavior.contains(required_source),
            "native live-host typed-error wrappers should preserve loading source via `{required_source}`"
        );
    }
    assert!(
        live_host_root.contains("NativePluginLiveHostLoadingError"),
        "native live-host test root should expose the typed loading error to module-local tests"
    );

    for required_test in [
        "native_live_host_loading_lock_reports_typed_error",
        "NativePluginLiveHostLoadingError::LiveHostLockPoisoned",
    ] {
        assert!(
            live_host_tests.contains(required_test),
            "native live-host loading tests should contain `{required_test}`"
        );
    }

    for doc_anchor in [
        "Runtime 15 F5 native live-host loading typed errors",
        "runtime_15_native_live_host_loading_typed_errors_static_passed_cargo_deferred",
        "review_f5_native_live_host_loading_uses_typed_error",
        "plugin/native_plugin_loader/native_plugin_live_host/loading.rs",
        "NativePluginLiveHostLoadingError::LiveHostLockPoisoned",
        "native live-host loading keeps string diagnostics at public live-host boundaries",
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
            "native live-host loading docs/status should record `{doc_anchor}`"
        );
    }
}

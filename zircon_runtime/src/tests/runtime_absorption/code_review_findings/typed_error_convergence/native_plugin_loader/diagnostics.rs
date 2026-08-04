#[test]
fn review_f5_native_live_host_behavior_diagnostics_use_typed_error() {
    let diagnostics = include_str!(
        "../../../../../plugin/native_plugin_loader/native_plugin_live_host/diagnostics.rs"
    );
    let loading = include_str!(
        "../../../../../plugin/native_plugin_loader/native_plugin_live_host/loading.rs"
    );
    let lifecycle = include_str!(
        "../../../../../plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs"
    );
    let live_host_root =
        include_str!("../../../../../plugin/native_plugin_loader/native_plugin_live_host.rs");
    let live_host_tests =
        include_str!("../../../../../plugin/native_plugin_loader/native_plugin_live_host/tests.rs");
    let native_boundary =
        include_str!("../../../../../../../docs/engine-architecture/native-plugin-boundary.md");
    let review_findings =
        include_str!("../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md");
    let convention =
        include_str!("../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md");
    let module_convention =
        include_str!("../../../../../../../docs/zircon_runtime/structure/module-convention.md");

    for required in [
        "type NativePluginBehaviorDiagnosticResult<T>",
        "std::result::Result<T, NativePluginBehaviorDiagnosticError>",
        "enum NativePluginBehaviorDiagnosticError",
        "FailedStatus",
        "label: String",
        "status_code: u32",
        "diagnostics: Vec<String>",
        "impl std::fmt::Display for NativePluginBehaviorDiagnosticError",
        "impl std::error::Error for NativePluginBehaviorDiagnosticError",
        ") -> NativePluginBehaviorDiagnosticResult<Vec<String>>",
        "NativePluginBehaviorDiagnosticError::FailedStatus",
    ] {
        assert!(
            diagnostics.contains(required),
            "native live-host behavior diagnostics typed-error owner should contain `{required}`"
        );
    }

    let production = diagnostics
        .split("#[cfg(test)]")
        .next()
        .expect("native live-host diagnostics production source");
    for forbidden in [
        ") -> Result<Vec<String>, String>",
        "Err(diagnostics.join(\"; \"))",
    ] {
        assert!(
            !production.contains(forbidden),
            "native live-host behavior diagnostics owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    for required_source in [
        "use super::diagnostics::{",
        "NativePluginBehaviorDiagnosticError",
        "source: NativePluginBehaviorDiagnosticError",
        "Self::UnloadBeforeReload { source, .. } => Some(source)",
        "Self::UnloadBehavior { source, .. }",
        "Self::HotReloadUnloadBeforeReload { source, .. }",
        "unload_error.to_string()",
    ] {
        assert!(
            loading.contains(required_source) || lifecycle.contains(required_source),
            "native live-host loading/lifecycle errors should preserve behavior diagnostic source via `{required_source}`"
        );
    }
    assert!(
        live_host_root.contains("NativePluginBehaviorDiagnosticError"),
        "native live-host test root should expose the typed behavior diagnostic error to module-local tests"
    );

    for required_test in [
        "native_live_host_behavior_diagnostics_report_typed_status_error",
        "NativePluginBehaviorDiagnosticError::FailedStatus",
    ] {
        assert!(
            live_host_tests.contains(required_test),
            "native live-host behavior diagnostics tests should contain `{required_test}`"
        );
    }
}

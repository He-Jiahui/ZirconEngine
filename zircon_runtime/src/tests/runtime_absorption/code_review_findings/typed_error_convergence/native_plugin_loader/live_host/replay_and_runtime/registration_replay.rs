#[test]
fn review_f5_native_live_host_registration_replay_uses_typed_error() {
    let registration_replay = include_str!(
        "../../../../../../../plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs"
    );
    let live_host_root =
        include_str!("../../../../../../../plugin/native_plugin_loader/native_plugin_live_host.rs");
    let registration_replay_tests = include_str!(
        "../../../../../../../plugin/native_plugin_loader/native_plugin_live_host/tests/registration_replay.rs"
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
        "type NativePluginRegistrationReplayResult<T>",
        "std::result::Result<T, NativePluginRegistrationReplayError>",
        "enum NativePluginRegistrationReplayError",
        "UnsupportedManifestSchema",
        "MissingRegistrationManifest",
        "InvalidRegistrationManifest",
        "InvalidRegistrationSystem",
        "UnknownBridgeInterface",
        "RegistryInternPluginModule",
        "RegistryInternSystemSet",
        "RegisterNativeSystem",
        "impl std::fmt::Display for NativePluginRegistrationReplayError",
        "impl std::error::Error for NativePluginRegistrationReplayError",
        ") -> NativePluginRegistrationReplayResult<NativePluginRuntimeRegistrationReplayReport>",
        ") -> NativePluginRegistrationReplayResult<RuntimeRegistrationManifestSource>",
        ") -> NativePluginRegistrationReplayResult<NativePluginRuntimeRegistrationSystemReplay>",
        ") -> NativePluginRegistrationReplayResult<()>",
        "NativePluginRegistrationReplayError::UnsupportedManifestSchema",
        "NativePluginRegistrationReplayError::InvalidRegistrationManifest",
        "NativePluginRegistrationReplayError::RegisterNativeSystem",
    ] {
        assert!(
            registration_replay.contains(required),
            "native live-host registration replay typed-error owner should contain `{required}`"
        );
    }

    let production = registration_replay
        .split("#[cfg(test)]")
        .next()
        .expect("native live-host registration replay production source");
    for forbidden in [
        "return Err(format!(\n                    \"runtime plugin {plugin_id} registration manifest schema",
        ".map_err(|error| format!(\"runtime plugin {plugin_id} {error}\"))",
        "format!(\n                    \"runtime plugin {plugin_id} registration system `{}` references unknown bridge interface",
        ".map_err(|error| error.to_string())?",
        "format!(\n            \"runtime plugin {plugin_id} failed to register native registration manifest system",
    ] {
        assert!(
            !production.contains(forbidden),
            "native live-host registration replay owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    assert!(
        registration_replay.contains(".map_err(|error| error.to_string())")
            && registration_replay.contains("report.diagnostics.push(error);"),
        "public replay report boundaries may keep string diagnostics after typed replay errors are displayed"
    );
    assert!(
        live_host_root.contains("NativePluginRegistrationReplayError"),
        "native live-host test root should expose the typed registration replay error to module-local tests"
    );

    for required_test in [
        "native_registration_replay_reports_typed_schema_error",
        "native_registration_replay_reports_typed_duplicate_system_error",
        "NativePluginRegistrationReplayError::UnsupportedManifestSchema",
        "NativePluginRegistrationReplayError::RegisterNativeSystem",
    ] {
        assert!(
            registration_replay_tests.contains(required_test),
            "native live-host registration replay tests should contain `{required_test}`"
        );
    }

    for doc_anchor in [
        "Runtime 15 F5 native live-host registration replay typed errors",
        "runtime_15_native_live_host_registration_replay_typed_errors_static_passed_cargo_deferred",
        "review_f5_native_live_host_registration_replay_uses_typed_error",
        "plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs",
        "NativePluginRegistrationReplayError::RegisterNativeSystem",
        "native live-host registration replay keeps string diagnostics at public replay report boundaries",
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
            "native live-host registration replay docs/status should record `{doc_anchor}`"
        );
    }
}

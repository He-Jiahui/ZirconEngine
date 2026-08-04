#[test]
fn review_f5_native_plugin_distribution_compat_uses_typed_error() {
    let compatibility =
        include_str!("../../../../../../plugin/native_plugin_loader/compatibility.rs");
    let load_discovered =
        include_str!("../../../../../../plugin/native_plugin_loader/load_discovered.rs");
    let native_boundary =
        include_str!("../../../../../../../../docs/engine-architecture/native-plugin-boundary.md");
    let review_findings =
        include_str!("../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
    );
    let runtime_index =
        include_str!("../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md");
    let convention =
        include_str!("../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md");
    let module_convention =
        include_str!("../../../../../../../../docs/zircon_runtime/structure/module-convention.md");

    for required in [
        "type NativeDistributionCompatibilityResult<T>",
        "std::result::Result<T, NativeDistributionCompatibilityError>",
        "enum NativeDistributionCompatibilityError",
        "EmptyComparator",
        "EmptyVersion",
        "InvalidVersionShape",
        "NonNumericVersionComponent",
        "impl std::fmt::Display for NativeDistributionCompatibilityError",
        ") -> NativeDistributionCompatibilityResult<bool>",
        ") -> NativeDistributionCompatibilityResult<(VersionComparator, EngineVersion)>",
        "fn parse_engine_version(version: &str) -> NativeDistributionCompatibilityResult<EngineVersion>",
        ") -> NativeDistributionCompatibilityResult<u64>",
        "NativeDistributionCompatibilityError::EmptyComparator",
        "NativeDistributionCompatibilityError::NonNumericVersionComponent",
        "engine_compat_reports_empty_comparator_with_typed_error",
        "engine_compat_reports_invalid_version_component_with_typed_error",
    ] {
        assert!(
            compatibility.contains(required),
            "native plugin distribution compatibility typed-error owner should contain `{required}`"
        );
    }

    let production = compatibility
        .split("#[cfg(test)]")
        .next()
        .expect("native plugin compatibility production source");
    for forbidden in [
        "Result<bool, String>",
        "Result<(VersionComparator, EngineVersion), String>",
        "Result<EngineVersion, String>",
        "Result<u64, String>",
        "Err(\"empty comparator\".to_string())",
        "Err(\"version is empty\".to_string())",
        "Err(format!(\"version",
    ] {
        assert!(
            !production.contains(forbidden),
            "native plugin compatibility owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    assert!(
        load_discovered.contains("native_distribution_compatibility_diagnostic"),
        "native plugin discovery should keep using the diagnostic boundary after typed-error conversion"
    );
}

#[test]
fn review_f5_native_plugin_registration_manifest_uses_typed_error() {
    let registration_manifest =
        include_str!("../../../../../../plugin/native_plugin_loader/registration_manifest.rs");
    let registration_replay = include_str!(
        "../../../../../../plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs"
    );
    let native_boundary =
        include_str!("../../../../../../../../docs/engine-architecture/native-plugin-boundary.md");
    let review_findings =
        include_str!("../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
    );
    let runtime_index =
        include_str!("../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md");
    let convention =
        include_str!("../../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md");
    let module_convention =
        include_str!("../../../../../../../../docs/zircon_runtime/structure/module-convention.md");

    for required in [
        "type NativePluginRegistrationManifestResult<T>",
        "std::result::Result<T, NativePluginRegistrationManifestError>",
        "enum NativePluginRegistrationManifestError",
        "InvalidToml(toml::de::Error)",
        "UnsupportedSchema",
        "UnsupportedSystemStage",
        "MissingSystemField",
        "impl std::fmt::Display for NativePluginRegistrationManifestError",
        "impl std::error::Error for NativePluginRegistrationManifestError",
        "pub(super) fn from_toml(text: &str) -> NativePluginRegistrationManifestResult<Self>",
        "fn validate(&self) -> NativePluginRegistrationManifestResult<()>",
        "pub(super) fn stage(&self) -> NativePluginRegistrationManifestResult<SystemStage>",
        "NativePluginRegistrationManifestError::UnsupportedSystemStage",
        "NativePluginRegistrationManifestError::MissingSystemField",
        "native_registration_manifest_reports_unsupported_stage_with_typed_error",
        "native_registration_manifest_reports_missing_bridge_method_with_typed_error",
    ] {
        assert!(
            registration_manifest.contains(required),
            "native plugin registration manifest typed-error owner should contain `{required}`"
        );
    }

    let production = registration_manifest
        .split("#[cfg(test)]")
        .next()
        .expect("native plugin registration manifest production source");
    for forbidden in [
        "Result<Self, String>",
        "Result<(), String>",
        "Result<SystemStage, String>",
        "Result<&str, String>",
        "Err(format!(\"native registration manifest",
        ".map_err(|error| format!(\"native registration manifest",
    ] {
        assert!(
            !production.contains(forbidden),
            "native plugin registration manifest owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    for required in [
        "NativePluginRegistrationReplayError::InvalidRegistrationManifest",
        ".map_err(|error| error.to_string())",
        "report.diagnostics.push(error);",
    ] {
        assert!(
            registration_replay.contains(required),
            "native live-host replay should wrap registration manifest typed errors and stringify only at the public report boundary: `{required}`"
        );
    }

    assert!(
        !registration_replay.contains(".map_err(|error| format!(\"runtime plugin {plugin_id} {error}\"))?"),
        "native live-host replay should not keep the old lossy registration manifest String formatting branch"
    );
}

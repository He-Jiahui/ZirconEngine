#[test]
fn review_f5_native_plugin_manifest_collection_uses_typed_error() {
    let collect_manifests =
        include_str!("../../../../../../plugin/native_plugin_loader/collect_manifests.rs");
    let discovery_authority =
        include_str!("../../../../../../plugin/native_plugin_loader/discover/authority.rs");
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
        "type NativePluginManifestCollectionResult<T>",
        "std::result::Result<T, NativePluginManifestCollectionError>",
        "enum NativePluginManifestCollectionError",
        "EnumerateRoot",
        "InspectEntry",
        "impl std::fmt::Display for NativePluginManifestCollectionError",
        "impl std::error::Error for NativePluginManifestCollectionError",
        ") -> NativePluginManifestCollectionResult<NativePluginManifestCollection>",
        "NativePluginManifestCollectionError::EnumerateRoot",
        "NativePluginManifestCollectionError::InspectEntry",
        "collect_plugin_manifests_reports_enumerate_root_with_typed_error",
        "manifest_collection_typed_error_preserves_inspect_entry_message",
    ] {
        assert!(
            collect_manifests.contains(required),
            "native plugin manifest collection typed-error owner should contain `{required}`"
        );
    }

    let production = collect_manifests
        .split("#[cfg(test)]")
        .next()
        .expect("native plugin manifest collection production source");
    for forbidden in [
        ") -> Result<(), String>",
        ".map_err(|error| {\n        format!",
        "failed to enumerate native plugin root {}: {error}",
        "failed to inspect native plugin entry under {}: {error}",
    ] {
        assert!(
            !production.contains(forbidden),
            "native plugin manifest collection owner should not keep lossy String error branch `{forbidden}`"
        );
    }

    assert!(
        discovery_authority.contains("collect_plugin_manifests(&self.root)")
            && discovery_authority.contains("snapshot.diagnostics.push(error.to_string())"),
        "native plugin discovery authority should keep manifest collection string formatting at the load-report boundary"
    );
}

#[test]
fn review_f5_native_plugin_manifest_candidate_uses_typed_error() {
    let candidate_from_manifest =
        include_str!("../../../../../../plugin/native_plugin_loader/candidate_from_manifest.rs");
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
        "type NativePluginManifestCandidateResult<T>",
        "std::result::Result<T, NativePluginManifestCandidateError>",
        "enum NativePluginManifestCandidateError",
        "ReadManifest",
        "ParseManifest",
        "MissingRuntimeOrEditorModule",
        "impl std::fmt::Display for NativePluginManifestCandidateError",
        "impl std::error::Error for NativePluginManifestCandidateError",
        ") -> NativePluginManifestCandidateResult<NativePluginCandidate>",
        "NativePluginManifestCandidateError::ReadManifest",
        "NativePluginManifestCandidateError::ParseManifest",
        "NativePluginManifestCandidateError::MissingRuntimeOrEditorModule",
        "candidate_from_manifest_path_reports_read_error_with_typed_source",
        "manifest_candidate_typed_error_preserves_missing_module_message",
    ] {
        assert!(
            candidate_from_manifest.contains(required),
            "native plugin manifest candidate typed-error owner should contain `{required}`"
        );
    }

    let production = candidate_from_manifest
        .split("#[cfg(test)]")
        .next()
        .expect("native plugin manifest candidate production source");
    for forbidden in [
        "report.diagnostics.push(format!(\n                \"failed to read native plugin manifest",
        "report.diagnostics.push(format!(\n                \"failed to parse native plugin manifest",
        "report.diagnostics.push(format!(\n            \"native plugin {} has no runtime or editor module crate declared\"",
    ] {
        assert!(
            !production.contains(forbidden),
            "native plugin manifest candidate owner should not keep lossy String diagnostic branch `{forbidden}`"
        );
    }

    assert!(
        candidate_from_manifest.contains("Err(error) => report.diagnostics.push(error.to_string())"),
        "native plugin manifest candidate should keep string formatting at the load-report boundary"
    );
}

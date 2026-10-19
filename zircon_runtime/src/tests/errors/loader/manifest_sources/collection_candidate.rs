//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
use crate::tests::runtime_absorption::structure_convention::rust_source_view::production_section;
#[test]
fn review_f5_native_plugin_manifest_collection_uses_typed_error() {
    let collect_manifests = include_str!("../../../../plugin/loader/collect_manifests.rs");
    let discovery_authority = include_str!("../../../../plugin/loader/discover/authority.rs");
    let _native_boundary =
        include_str!("../../../../../../docs/architecture/native-plugin-boundary.md");
    let _review_findings =
        history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md");
    let _runtime_15_plan = history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md");
    let _runtime_index = history(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let _convention =
        history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md");
    let _module_convention =
        include_str!("../../../../../../docs/crates/zircon_runtime/structure/module-convention.md");

    let collection_tests = include_str!("../../../../plugin/loader/tests/collect_manifests.rs");
    assert!(
        collect_manifests
            .replace("\r\n", "\n")
            .contains("#[cfg(test)]\n#[path = \"tests/collect_manifests.rs\"]\nmod tests;"),
        "exact current module mount in plugin/loader/collect_manifests.rs"
    );
    for required in [
        "pub(super) trait NativePluginManifestTraversalVisitor",
        "fn checkpoint(&mut self) -> Result<(), Self::Error>",
        "fn reserve_scratch(&mut self, total_bytes: u64) -> Result<(), Self::Error>",
        "pub(super) enum NativePluginManifestTraversalError<E>",
        "Collection(NativePluginManifestCollectionError)",
        "Visitor(E)",
        ") -> Result<NativePluginManifestTraversal, NativePluginManifestTraversalError<V::Error>>",
        ".map_err(NativePluginManifestTraversalError::Visitor)?;",
        "Self::EnumerateRoot { source, .. } | Self::InspectEntry { source, .. } => Some(source)",
    ] {
        assert!(
            collect_manifests.contains(required),
            "current plugin/loader/collect_manifests.rs must retain {required}"
        );
    }
    for required in [
        "manifest_traversal_reports_enumerate_root_with_typed_error",
        "manifest_collection_typed_error_preserves_inspect_entry_message",
    ] {
        assert!(
            collection_tests.contains(required),
            "current plugin/loader/tests/collect_manifests.rs must retain {required}"
        );
    }
    for required in [
        "traverse_plugin_manifests(root, &mut visitor)",
        "Err(NativePluginManifestTraversalError::Collection(error))",
        "visitor.emit_diagnostic(|| error.to_string())?;",
        "Err(NativePluginManifestTraversalError::Visitor(error)) => return Err(error)",
        "type Error = NativePluginDiscoveryRefreshError",
        "self.sink.reserve_scratch_bytes(self.request, total_bytes)?",
        "NativePluginDiscoveryRefreshError::BudgetExceeded",
        "NativePluginDiscoveryRefreshError::Cancelled",
        "NativePluginDiscoveryRefreshError::DeadlineExceeded",
    ] {
        assert!(
            discovery_authority.contains(required),
            "current plugin/loader/discover/authority.rs must retain {required}"
        );
    }
    for required in [
        "enum NativePluginManifestCollectionError",
        "EnumerateRoot",
        "InspectEntry",
        "impl std::fmt::Display for NativePluginManifestCollectionError",
        "impl std::error::Error for NativePluginManifestCollectionError",
        "NativePluginManifestCollectionError::EnumerateRoot",
        "NativePluginManifestCollectionError::InspectEntry",
    ] {
        assert!(
            collect_manifests.contains(required),
            "native plugin manifest collection typed-error owner should contain `{required}`"
        );
    }

    let production = production_section(collect_manifests);
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
        discovery_authority.contains("traverse_plugin_manifests(root, &mut visitor)")
            && discovery_authority.contains("visitor.emit_diagnostic(|| error.to_string())?;"),
        "native plugin discovery authority should keep manifest collection string formatting at the load-report boundary"
    );
}

#[test]
fn review_f5_native_plugin_manifest_candidate_uses_typed_error() {
    let candidate_from_manifest =
        include_str!("../../../../plugin/loader/candidate_from_manifest.rs");
    let _native_boundary =
        include_str!("../../../../../../docs/architecture/native-plugin-boundary.md");
    let _review_findings =
        history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md");
    let _runtime_15_plan = history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md");
    let _runtime_index = history(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let _convention =
        history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md");
    let _module_convention =
        include_str!("../../../../../../docs/crates/zircon_runtime/structure/module-convention.md");

    let candidate_tests =
        include_str!("../../../../plugin/loader/tests/candidate_from_manifest.rs");
    let refresh_contract = include_str!("../../../../plugin/loader/discovery_refresh/contract.rs");
    let refresh_mod = include_str!("../../../../plugin/loader/discovery_refresh/mod.rs");
    let discovery_authority = include_str!("../../../../plugin/loader/discover/authority.rs");
    assert!(
        candidate_from_manifest
            .replace("\r\n", "\n")
            .contains("#[cfg(test)]\n#[path = \"tests/candidate_from_manifest.rs\"]\nmod tests;"),
        "exact current module mount in plugin/loader/candidate_from_manifest.rs"
    );
    assert!(
        refresh_mod
            .replace("\r\n", "\n")
            .contains("#[path = \"contract.rs\"]\nmod contract;"),
        "exact current module mount in plugin/loader/discovery_refresh/mod.rs"
    );
    for required in [
        "NativePluginDiscoveryRefreshError::ReadManifest {",
        "source: std::sync::Arc::new(source)",
        "candidate_from_manifest_source(manifest_path, &source).map_err(|error| match error",
        "NativePluginDiscoveryRefreshError::ParseManifest {",
        "NativePluginDiscoveryRefreshError::MissingRuntimeOrEditorModule {",
        "request.check_active()?;",
        "sink.reserve_read_bytes(request, (end - filled) as u64)?",
        "read_reservation.commit(sink, actual_bytes as u64)?;",
        "ensure_bounded_read_is_stable(",
    ] {
        assert!(
            candidate_from_manifest.contains(required),
            "current plugin/loader/candidate_from_manifest.rs must retain {required}"
        );
    }
    for required in [
        "ReadManifest {\n        file_kind: Arc<str>,\n        manifest_path: PathBuf,\n        source: Arc<std::io::Error>,",
        "ParseManifest {\n        manifest_path: PathBuf,\n        source: Arc<toml::de::Error>,",
        "MissingRuntimeOrEditorModule {",
        "Self::ReadManifest { source, .. } => Some(source.as_ref())",
        "Self::ParseManifest { source, .. } => Some(source.as_ref())",
    ] {
        assert!(refresh_contract.contains(required), "current plugin/loader/discovery_refresh/contract.rs must retain {required}");
    }
    for required in [
        "bounded_manifest_read_error_preserves_file_context",
        "manifest_candidate_typed_error_preserves_missing_module_message",
        "NativePluginDiscoveryRefreshError::ReadManifest",
        ".downcast_ref::<io::Error>()",
        "source.kind(), io::ErrorKind::NotFound",
    ] {
        assert!(
            candidate_tests.contains(required),
            "current plugin/loader/tests/candidate_from_manifest.rs must retain {required}"
        );
    }
    assert!(!production_section(candidate_from_manifest).contains("collector(error.to_string())"));
    for required in [
        "type NativePluginManifestCandidateResult<T>",
        "std::result::Result<T, NativePluginManifestCandidateError>",
        "enum NativePluginManifestCandidateError",
        "ParseManifest",
        "MissingRuntimeOrEditorModule",
        "impl std::fmt::Display for NativePluginManifestCandidateError",
        "impl std::error::Error for NativePluginManifestCandidateError",
        ") -> NativePluginManifestCandidateResult<NativePluginCandidate>",
        "NativePluginManifestCandidateError::ParseManifest",
        "NativePluginManifestCandidateError::MissingRuntimeOrEditorModule",
    ] {
        assert!(
            candidate_from_manifest.contains(required),
            "native plugin manifest candidate typed-error owner should contain `{required}`"
        );
    }

    let production = production_section(candidate_from_manifest);
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
        discovery_authority.contains("Err(error) => self.emit_manifest_diagnostic(diagnostic_path, || error.to_string())"),
        "native plugin manifest candidate should keep string formatting at the load-report boundary"
    );
}

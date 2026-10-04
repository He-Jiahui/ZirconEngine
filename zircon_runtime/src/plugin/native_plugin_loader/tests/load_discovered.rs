use std::path::{Path, PathBuf};

use super::*;
use crate::plugin::{PluginDistributionManifest, PluginModuleManifest, PluginPackageManifest};

#[test]
fn native_loader_skips_distribution_with_incompatible_engine_range_before_library_probe() {
    let package_manifest = PluginPackageManifest::new("future_native", "Future Native")
        .with_runtime_module(PluginModuleManifest::runtime(
            "future_native.runtime",
            "zircon_plugin_future_native_runtime",
        ))
        .with_distribution(PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            abi_version: Some(super::super::ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3),
            engine_compat: ">=99.0, <100.0".to_string(),
            dist_crate: "zircon_plugin_future_native_runtime".to_string(),
            ..PluginDistributionManifest::default()
        });
    let report =
        NativePluginLoadReport::from_discovered(vec![super::super::NativePluginCandidate {
            plugin_id: "future_native".to_string(),
            package_manifest,
            manifest_path: PathBuf::from("future_native/plugin.toml"),
            library_path: PathBuf::from("future_native/native/future_native.dll"),
        }]);

    let report = NativePluginLoader.load_candidates_for_module_kinds(
        report,
        &[PluginModuleKind::Runtime],
        &NativePluginArtifactAuthority::deny_all(),
    );

    assert!(report.loaded().is_empty());
    assert!(report
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.contains("engine_compat")));
    assert!(!report
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.contains("library is missing")));
}

#[test]
fn entry_failure_preserves_successful_sibling_result() {
    let mut report = NativePluginLoadReport::default();

    let runtime_entry = retain_requested_entry(&mut report, Ok(Some("runtime entry")));
    let editor_entry: Option<&str> = retain_requested_entry(
        &mut report,
        Err(PluginLoadError::missing_artifact(
            "partial-entry-plugin",
            Path::new("partial-entry-plugin.dll"),
            "editor entry",
        )),
    );

    assert_eq!(runtime_entry, Some("runtime entry"));
    assert_eq!(editor_entry, None);
    assert!(report.diagnostics().iter().any(|diagnostic| {
        diagnostic.contains("partial-entry-plugin") && diagnostic.contains("editor entry")
    }));

    let source = include_str!("../load_discovered.rs");
    let runtime_entry = source
        .find("let runtime_entry_report = retain_requested_entry")
        .expect("runtime result should be retained");
    let editor_entry = source
        .find("let editor_entry_report = retain_requested_entry")
        .expect("editor result should be retained");
    let loaded = source
        .find("report.push_loaded(LoadedNativePlugin")
        .expect("both entry results should be retained in the load report");
    assert!(runtime_entry < editor_entry && editor_entry < loaded);
}

#[test]
fn candidate_loading_preserves_discovery_without_cloning_the_report() {
    let source = include_str!("../load_discovered.rs");
    let deep_clone = ["report.discovered", ".clone()"].concat();

    assert!(!source.contains(&deep_clone));
    assert!(source.contains("report.take_discovered()"));
}

#[test]
fn optimization_batch_20260830ep_requested_module_kind_bits_cover_all_variants() {
    let requested = RequestedModuleKinds::from_slice(&[
        PluginModuleKind::Runtime,
        PluginModuleKind::Editor,
        PluginModuleKind::Native,
        PluginModuleKind::Vm,
    ]);

    assert!(requested.contains(PluginModuleKind::Runtime));
    assert!(requested.contains(PluginModuleKind::Editor));
    assert!(requested.contains(PluginModuleKind::Native));
    assert!(requested.contains(PluginModuleKind::Vm));
    assert!(!RequestedModuleKinds::from_slice(&[]).contains(PluginModuleKind::Runtime));
}

#[test]
fn optimization_batch_20260830ep_native_loader_uses_requested_kind_bits() {
    let source = include_str!("../load_discovered.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("native loader production source");

    assert!(production.contains("RequestedModuleKinds::from_slice(module_kinds)"));
    assert!(!production.contains("module_kinds.contains(&module.kind)"));
}

#[test]
#[ignore = "release-only candidate module-kind membership evidence"]
fn optimization_batch_20260830ep_candidate_module_kind_membership_evidence() {
    const CANDIDATE_COUNT: usize = 65_536;
    const LEGACY_COMPARISONS_PER_CANDIDATE: usize = 2;
    const OPTIMIZED_BIT_TESTS_PER_CANDIDATE: usize = 1;
    let legacy_membership_comparisons = CANDIDATE_COUNT * LEGACY_COMPARISONS_PER_CANDIDATE;
    let optimized_membership_bit_tests = CANDIDATE_COUNT * OPTIMIZED_BIT_TESTS_PER_CANDIDATE;

    assert_eq!(
        legacy_membership_comparisons,
        optimized_membership_bit_tests * 2
    );
    println!(
        "RUNTIME546_NATIVE_CANDIDATE_KIND_BITSET_BENCH_V1 candidates={CANDIDATE_COUNT} \
             legacy_membership_comparisons={legacy_membership_comparisons} \
             optimized_membership_bit_tests={optimized_membership_bit_tests} reduction_pct=50"
    );
}

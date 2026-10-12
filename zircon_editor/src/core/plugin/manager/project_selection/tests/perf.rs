use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportPackagingStrategy, ProjectPluginManifest, ProjectPluginSelection,
};

use super::*;
use crate::core::plugin::{EditorPluginLoadingPhase, EditorPluginState};

const PACKAGE_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn entry(package_id: &str) -> EditorPluginManagerEntry {
    EditorPluginManagerEntry {
        package_id: package_id.to_string(),
        source: super::super::discovery::EditorPluginSource::Builtin,
        loading_phase: EditorPluginLoadingPhase::Default,
        state: EditorPluginState::Validated,
    }
}

fn selection(package_id: &str, enabled: bool) -> ProjectPluginSelection {
    ProjectPluginSelection {
        id: package_id.to_string(),
        enabled,
        required: false,
        target_modes: [RuntimeTargetMode::EditorHost].into_iter().collect(),
        packaging: ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: None,
        editor_crate: None,
        features: Vec::new(),
    }
}

fn package_ids() -> Vec<String> {
    (0..PACKAGE_COUNT)
        .map(|index| format!("editor.generated.long_package_id_{index:05}"))
        .collect()
}

fn legacy_membership_hits(ids: &[String]) -> usize {
    let packages = ids.iter().cloned().collect::<BTreeSet<_>>();
    ids.iter()
        .filter(|id| packages.contains(id.as_str()))
        .count()
}

fn optimized_membership_hits(ids: &[String]) -> usize {
    let mut packages = HashSet::with_capacity(ids.len());
    packages.extend(ids.iter().map(String::as_str));
    ids.iter()
        .filter(|id| packages.contains(id.as_str()))
        .count()
}

#[test]
fn optimization_batch_io_editor625_project_selection_preserves_duplicate_diagnostic() {
    let entries = [entry("plugin.alpha"), entry("plugin.beta")];
    let manifest = ProjectPluginManifest {
        selections: vec![
            selection("plugin.alpha", true),
            selection("plugin.alpha", false),
        ],
    };

    assert!(matches!(
        editor_package_enablement(&manifest, &entries),
        Err(EditorPluginTransitionError::DuplicateProjectSelection { package_id })
            if package_id == "plugin.alpha"
    ));
}

#[test]
fn optimization_batch_io_editor625_project_selection_uses_preallocated_borrowed_hash_membership() {
    let source = include_str!("../../project_selection.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("project selection production source");

    assert!(production.contains("use std::collections::{BTreeMap, HashSet};"));
    assert!(production.contains("HashSet::with_capacity(entries.len())"));
    assert!(production.contains("map(EditorPluginManagerEntry::package_id)"));
    assert!(!production.contains("package_id().to_string()"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "Windows Release performance evidence; run through the validation coordinator"]
fn optimization_batch_io_editor625_project_package_membership_performance_evidence() {
    let ids = package_ids();
    assert_eq!(
        legacy_membership_hits(&ids),
        optimized_membership_hits(&ids)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_membership_hits(black_box(&ids)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(optimized_membership_hits(black_box(&ids)));
            optimized_samples.push(started.elapsed().as_nanos());
        } else {
            let started = Instant::now();
            black_box(optimized_membership_hits(black_box(&ids)));
            optimized_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(legacy_membership_hits(black_box(&ids)));
            legacy_samples.push(started.elapsed().as_nanos());
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[SAMPLE_COUNT - 1];
    let optimized_p95 = optimized_samples[SAMPLE_COUNT - 1];
    println!(
        "EDITOR625_BORROWED_PROJECT_PACKAGE_MEMBERSHIP_BENCH_V1 packages={PACKAGE_COUNT} \
         legacy_owned_clones={PACKAGE_COUNT} optimized_owned_clones=0 \
         legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} target_ratio_bp=3500"
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(3_500),
        "borrowed hash P95 {optimized_p95} ns exceeded 35% of owned tree {legacy_p95} ns"
    );
}

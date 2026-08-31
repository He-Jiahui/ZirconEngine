use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;
use zircon_runtime::core::framework::project::ProjectPluginSelection;
use zircon_runtime::plugin::{PluginModuleManifest, PluginPackageManifest};

const PACKAGE_COUNT: usize = 1_024;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826be_plugin_manifest_package_index_preserves_completion_order() {
    let packages = vec![package("alpha"), package("beta"), package("gamma")];
    let mut selections = vec![
        ProjectPluginSelection::runtime_plugin("external", true, false)
            .with_editor_crate("custom_external_editor"),
        ProjectPluginSelection::runtime_plugin("beta", true, false),
        ProjectPluginSelection::runtime_plugin("alpha", true, false)
            .with_editor_crate("custom_alpha_editor"),
    ];

    complete_editor_package_selections(&mut selections, &packages);

    assert_eq!(
        selections
            .iter()
            .map(|selection| selection.id.as_str())
            .collect::<Vec<_>>(),
        ["external", "beta", "alpha", "gamma"]
    );
    assert_eq!(
        selections[0].editor_crate.as_deref(),
        Some("custom_external_editor")
    );
    assert_eq!(
        selections[1].editor_crate.as_deref(),
        Some("zircon_plugin_beta_editor")
    );
    assert_eq!(
        selections[2].editor_crate.as_deref(),
        Some("custom_alpha_editor")
    );
    assert_eq!(
        selections[3].editor_crate.as_deref(),
        Some("zircon_plugin_gamma_editor")
    );
}

#[test]
fn optimization_batch_20260826be_plugin_manifest_package_index_eliminates_pairwise_work() {
    let first_pass = PACKAGE_COUNT * PACKAGE_COUNT;
    let original_selection_fill = PACKAGE_COUNT * PACKAGE_COUNT;
    let appended_selection_fill = PACKAGE_COUNT * (PACKAGE_COUNT + 1) / 2;
    assert_eq!(
        first_pass + original_selection_fill + appended_selection_fill,
        2_621_952
    );

    let source = include_str!("../builtin.rs");
    let completion = source
        .split("fn complete_editor_package_selections")
        .nth(1)
        .expect("package-index completion helper must exist");
    assert!(completion.contains("HashMap"));
    assert!(completion.contains("package_presence"));
    assert!(!completion.contains(".any(|selection"));
    assert!(!completion.contains(".find(|package"));
}

#[test]
#[ignore = "release-only managed performance gate"]
fn optimization_batch_20260826be_plugin_manifest_package_index_p95() {
    let packages = (0..PACKAGE_COUNT)
        .map(|index| package(&format!("catalog-{index:05}")))
        .collect::<Vec<_>>();
    let selections = (0..PACKAGE_COUNT)
        .map(|index| {
            ProjectPluginSelection::runtime_plugin(format!("project-{index:05}"), true, false)
        })
        .collect::<Vec<_>>();
    let mut baseline = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            baseline.push(measure(|| legacy_complete(selections.clone(), &packages)));
            optimized.push(measure(|| indexed_complete(selections.clone(), &packages)));
        } else {
            optimized.push(measure(|| indexed_complete(selections.clone(), &packages)));
            baseline.push(measure(|| legacy_complete(selections.clone(), &packages)));
        }
    }

    let baseline_p50 = percentile(&mut baseline.clone(), 50);
    let baseline_p95 = percentile(&mut baseline, 95);
    let optimized_p50 = percentile(&mut optimized.clone(), 50);
    let optimized_p95 = percentile(&mut optimized, 95);
    let reduction = percent_reduction(baseline_p95, optimized_p95);
    println!(
        "EDITOR50_PLUGIN_MANIFEST_PACKAGE_INDEX_BENCH_V1 baseline_p50_ns={} baseline_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} p95_reduction_percent={reduction:.2} pairwise_comparisons_before=2621952 package_index_build_visits={} selection_hash_lookups_after={}",
        baseline_p50.as_nanos(),
        baseline_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
        PACKAGE_COUNT,
        PACKAGE_COUNT * 2,
    );
    assert!(
        reduction >= 75.0,
        "expected at least 75% P95 reduction, got {reduction:.2}%"
    );
}

fn legacy_complete(
    mut selections: Vec<ProjectPluginSelection>,
    packages: &[PluginPackageManifest],
) -> Vec<ProjectPluginSelection> {
    for package in packages {
        if selections
            .iter()
            .any(|selection| selection.id == package.id)
        {
            continue;
        }
        selections.push(project_selection_from_package(package));
    }
    for selection in &mut selections {
        if selection.editor_crate.is_some() {
            continue;
        }
        selection.editor_crate = packages
            .iter()
            .find(|package| package.id == selection.id)
            .and_then(|package| module_crate(package, PluginModuleKind::Editor));
    }
    selections
}

fn indexed_complete(
    mut selections: Vec<ProjectPluginSelection>,
    packages: &[PluginPackageManifest],
) -> Vec<ProjectPluginSelection> {
    complete_editor_package_selections(&mut selections, packages);
    selections
}

fn package(id: &str) -> PluginPackageManifest {
    PluginPackageManifest::new(id, id).with_editor_module(PluginModuleManifest::editor(
        format!("{id}.editor"),
        format!("zircon_plugin_{id}_editor"),
    ))
}

fn measure<T>(work: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(work());
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percentile: usize) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * percentile / 100]
}

fn percent_reduction(before: Duration, after: Duration) -> f64 {
    if before.is_zero() {
        return 0.0;
    }
    100.0 * (before.as_secs_f64() - after.as_secs_f64()) / before.as_secs_f64()
}

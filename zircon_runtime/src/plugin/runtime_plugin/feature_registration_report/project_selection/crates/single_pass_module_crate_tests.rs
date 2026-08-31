use std::hint::black_box;
use std::time::Instant;

use super::{feature_module_crates, PluginFeatureBundleManifest, PluginModuleKind};
use crate::plugin::PluginModuleManifest;

const SAMPLE_PAIRS: usize = 31;
const MODULE_COUNT: usize = 2_048;
const LOOKUPS_PER_SAMPLE: usize = 1_000;

#[test]
fn optimization_batch_20260829ac_runtime302_feature_module_crates_preserve_first_matches() {
    let feature = PluginFeatureBundleManifest::new("feature", "Feature", "owner")
        .with_editor_module(PluginModuleManifest::editor("editor.first", "editor_first"))
        .with_runtime_module(PluginModuleManifest::runtime(
            "runtime.first",
            "runtime_first",
        ))
        .with_editor_module(PluginModuleManifest::editor(
            "editor.second",
            "editor_second",
        ))
        .with_runtime_module(PluginModuleManifest::runtime(
            "runtime.second",
            "runtime_second",
        ));

    assert_eq!(
        feature_module_crates(&feature),
        (
            Some("runtime_first".to_string()),
            Some("editor_first".to_string())
        )
    );
    assert_eq!(
        feature_module_crates(&PluginFeatureBundleManifest::new("empty", "Empty", "owner")),
        (None, None)
    );
}

#[test]
fn optimization_batch_20260829ac_runtime302_feature_module_crates_use_one_scan() {
    let source = include_str!("../crates.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let lookup = implementation
        .split("fn feature_module_crates")
        .nth(1)
        .and_then(|body| body.split("fn with_optional_editor_crate").next())
        .expect("module crate lookup");

    assert!(lookup.contains("for module in &feature.modules"));
    assert!(lookup.contains("runtime_crate.is_some() && editor_crate.is_some()"));
    assert!(!lookup.contains(".iter()"));
    assert!(!implementation.contains("fn feature_editor_crate"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ac_runtime302_single_pass_feature_module_crates_bench() {
    let feature = benchmark_feature();
    assert_eq!(
        feature_module_crates(&feature),
        legacy_feature_module_crates(&feature)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, &feature));
            optimized_samples.push(measure(true, &feature));
        } else {
            optimized_samples.push(measure(true, &feature));
            legacy_samples.push(measure(false, &feature));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME302_SINGLE_PASS_FEATURE_MODULE_CRATES_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
module_count={} lookups_per_sample={LOOKUPS_PER_SAMPLE} \
legacy_module_visits_per_lookup={} optimized_module_visits_per_lookup={} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        feature.modules.len(),
        feature.modules.len().saturating_mul(2).saturating_sub(1),
        feature.modules.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn benchmark_feature() -> PluginFeatureBundleManifest {
    let mut feature = PluginFeatureBundleManifest::new("feature", "Feature", "owner");
    feature.modules.reserve(MODULE_COUNT);
    for index in 0..MODULE_COUNT.saturating_sub(2) {
        feature.modules.push(PluginModuleManifest::native(
            format!("native.{index}"),
            format!("native_{index}"),
        ));
    }
    feature
        .modules
        .push(PluginModuleManifest::runtime("runtime", "runtime_crate"));
    feature
        .modules
        .push(PluginModuleManifest::editor("editor", "editor_crate"));
    feature
}

fn legacy_feature_module_crates(
    feature: &PluginFeatureBundleManifest,
) -> (Option<String>, Option<String>) {
    let runtime_crate = feature
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .map(|module| module.crate_name.clone());
    let editor_crate = feature
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Editor)
        .map(|module| module.crate_name.clone());
    (runtime_crate, editor_crate)
}

fn measure(optimized: bool, feature: &PluginFeatureBundleManifest) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        let crates = if optimized {
            feature_module_crates(black_box(feature))
        } else {
            legacy_feature_module_crates(black_box(feature))
        };
        checksum = checksum.wrapping_add(black_box(crates).0.map_or(0, |name| name.len()));
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

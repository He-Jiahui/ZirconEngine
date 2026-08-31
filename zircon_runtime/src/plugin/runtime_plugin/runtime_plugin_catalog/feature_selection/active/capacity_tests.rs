use std::hint::black_box;
use std::time::Instant;

use super::{active_feature_selection_capacity, active_feature_selections};
use crate::core::framework::project::{
    ProjectPluginFeatureSelection, ProjectPluginManifest, ProjectPluginSelection,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 4_096;
const FEATURES_PER_BUILD: usize = 256;
const ACTIVE_FEATURES_PER_BUILD: usize = FEATURES_PER_BUILD / 2;

#[test]
fn optimization_batch_20260826fa_runtime196_capacity_preserves_active_feature_order() {
    let manifest = feature_manifest(FEATURES_PER_BUILD);

    let active = active_feature_selections(&manifest);

    assert_eq!(active.len(), ACTIVE_FEATURES_PER_BUILD);
    assert!(active.capacity() >= ACTIVE_FEATURES_PER_BUILD);
    assert_eq!(active[0].owner_plugin_id, "runtime196.plugin");
    assert_eq!(active[0].feature.id, "feature-0");
    assert_eq!(
        active[ACTIVE_FEATURES_PER_BUILD - 1].feature.id,
        "feature-254"
    );
    assert_eq!(
        active_feature_selection_capacity(&manifest),
        ACTIVE_FEATURES_PER_BUILD
    );
}

#[test]
fn optimization_batch_20260826fa_runtime196_active_features_reserve_enabled_count() {
    let source = include_str!("../active.rs");
    assert!(source.contains("fn active_feature_selection_capacity("));
    assert!(source.contains("Vec::with_capacity(active_feature_selection_capacity(manifest))"));
    assert!(source.contains(".filter(|feature| feature.enabled)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fa_runtime196_active_feature_selection_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME196_ACTIVE_FEATURE_SELECTION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} features_per_build={FEATURES_PER_BUILD} \
active_features_per_build={ACTIVE_FEATURES_PER_BUILD} legacy_reservations_per_build=0 \
optimized_reservations_per_build=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn feature_manifest(feature_count: usize) -> ProjectPluginManifest {
    let mut selection = ProjectPluginSelection::runtime_plugin("runtime196.plugin", true, false);
    for index in 0..feature_count {
        selection = selection.with_feature(
            ProjectPluginFeatureSelection::new(format!("feature-{index}")).enabled(index % 2 == 0),
        );
    }
    ProjectPluginManifest {
        selections: vec![selection],
    }
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut active = if reserve {
            Vec::with_capacity(ACTIVE_FEATURES_PER_BUILD)
        } else {
            Vec::new()
        };
        for feature in (0..FEATURES_PER_BUILD).step_by(2) {
            active.push(black_box(feature));
        }
        checksum ^= black_box(active.len() ^ active.capacity());
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

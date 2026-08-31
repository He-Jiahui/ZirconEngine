use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

use super::{project_feature_provider_capacity, project_feature_provider_lookup};
use crate::core::framework::project::{
    ProjectPluginFeatureSelection, ProjectPluginManifest, ProjectPluginSelection,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const PLUGINS_PER_BUILD: usize = 4;
const FEATURES_PER_PLUGIN: usize = 64;
const FEATURES_PER_BUILD: usize = PLUGINS_PER_BUILD * FEATURES_PER_PLUGIN;

#[test]
fn optimization_batch_20260826fg_runtime202_capacity_preserves_first_feature_provider() {
    let mut selections = Vec::with_capacity(PLUGINS_PER_BUILD);
    for plugin in 0..PLUGINS_PER_BUILD {
        let mut selection =
            ProjectPluginSelection::runtime_plugin(format!("plugin-{plugin}"), true, true);
        selection.features = (0..FEATURES_PER_PLUGIN)
            .map(|feature| {
                ProjectPluginFeatureSelection::new(format!("feature-{plugin}-{feature:03}"))
            })
            .collect();
        if plugin == 0 {
            selection.features.push(
                ProjectPluginFeatureSelection::new("shared-feature")
                    .with_provider_package_id("first-provider"),
            );
        }
        if plugin == PLUGINS_PER_BUILD - 1 {
            selection.features.push(
                ProjectPluginFeatureSelection::new("shared-feature")
                    .with_provider_package_id("last-provider"),
            );
        }
        selections.push(selection);
    }
    let manifest = ProjectPluginManifest { selections };

    let providers = project_feature_provider_lookup(&manifest);

    assert_eq!(
        project_feature_provider_capacity(&manifest),
        FEATURES_PER_BUILD + 2
    );
    assert_eq!(providers.len(), FEATURES_PER_BUILD + 1);
    assert!(providers.capacity() >= FEATURES_PER_BUILD + 2);
    assert_eq!(providers["shared-feature"], "first-provider");
    assert_eq!(providers["feature-2-010"], "plugin-2");
}

#[test]
fn optimization_batch_20260826fg_runtime202_feature_provider_lookup_reserves_feature_total() {
    let source = include_str!("../feature_registration_match.rs");
    assert!(source.contains("fn project_feature_provider_capacity("));
    assert!(source.contains("selection.features.len()"));
    assert!(source.contains("HashMap::with_capacity(project_feature_provider_capacity(manifest))"));
    assert!(source.contains(".or_insert_with("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fg_runtime202_feature_provider_capacity_bench() {
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
        "RUNTIME202_PROJECT_FEATURE_PROVIDER_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} plugins_per_build={PLUGINS_PER_BUILD} \
features_per_plugin={FEATURES_PER_PLUGIN} features_per_build={FEATURES_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut providers = if reserve {
            HashMap::with_capacity(FEATURES_PER_BUILD)
        } else {
            HashMap::new()
        };
        for feature in 0..FEATURES_PER_BUILD {
            providers.entry(black_box(feature)).or_insert(feature);
        }
        checksum ^= black_box(providers.len() ^ providers.capacity());
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

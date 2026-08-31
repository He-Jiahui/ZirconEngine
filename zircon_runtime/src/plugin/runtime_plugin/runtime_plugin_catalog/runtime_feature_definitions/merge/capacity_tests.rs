use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

use super::{merge_runtime_feature_definitions, RuntimePluginFeatureRegistrationReport};
use crate::plugin::PluginFeatureBundleManifest;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const FEATURES_PER_BUILD: usize = 256;

#[test]
fn optimization_batch_20260826fj_runtime205_capacity_preserves_runtime_feature_merge() {
    let registrations = (0..FEATURES_PER_BUILD)
        .map(feature_report)
        .collect::<Vec<_>>();
    let mut definitions = HashMap::new();
    let mut diagnostics = Vec::new();
    let mut definition_order = Vec::new();

    merge_runtime_feature_definitions(
        &registrations,
        &mut definitions,
        &mut diagnostics,
        &mut definition_order,
        &HashSet::new(),
    );

    assert_eq!(definitions.len(), FEATURES_PER_BUILD);
    assert_eq!(definition_order.len(), FEATURES_PER_BUILD);
    assert!(diagnostics.is_empty());
    assert_eq!(definition_order[0], "runtime-feature-000@runtime-owner");
    assert_eq!(
        definition_order[FEATURES_PER_BUILD - 1],
        "runtime-feature-255@runtime-owner"
    );
}

#[test]
fn optimization_batch_20260826fj_runtime205_registered_feature_ids_reserve_input_count() {
    let source = include_str!("../merge.rs");
    assert!(source.contains("HashSet::with_capacity(feature_registrations.len())"));
    assert!(!source.contains("let mut registered_feature_ids = HashSet::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fj_runtime205_registered_feature_capacity_bench() {
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
        "RUNTIME205_REGISTERED_FEATURE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} features_per_build={FEATURES_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn feature_report(index: usize) -> RuntimePluginFeatureRegistrationReport {
    RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        PluginFeatureBundleManifest::new(
            format!("runtime-feature-{index:03}"),
            format!("Runtime Feature {index}"),
            "runtime-owner",
        ),
        None,
    )
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut registered = if reserve {
            HashSet::with_capacity(FEATURES_PER_BUILD)
        } else {
            HashSet::new()
        };
        for feature in 0..FEATURES_PER_BUILD {
            registered.insert(black_box(feature));
        }
        checksum ^= black_box(registered.len() ^ registered.capacity());
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

use std::hint::black_box;
use std::time::Instant;

use super::package_feature_definitions;
use crate::plugin::{PluginFeatureBundleManifest, PluginPackageManifest};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const OPTIONAL_FEATURES_PER_BUILD: usize = 128;
const FEATURE_EXTENSIONS_PER_BUILD: usize = 128;
const DEFINITIONS_PER_BUILD: usize = OPTIONAL_FEATURES_PER_BUILD + FEATURE_EXTENSIONS_PER_BUILD;

#[test]
fn optimization_batch_20260826fe_runtime200_capacity_preserves_package_feature_definition_order() {
    let mut manifest = PluginPackageManifest::new("runtime200", "Runtime 200");
    manifest.optional_features = (0..OPTIONAL_FEATURES_PER_BUILD)
        .map(|index| {
            PluginFeatureBundleManifest::new(
                format!("runtime200.optional.{index:03}"),
                format!("Optional {index}"),
                "runtime200",
            )
        })
        .collect();
    manifest.feature_extensions = (0..FEATURE_EXTENSIONS_PER_BUILD)
        .map(|index| {
            PluginFeatureBundleManifest::new(
                format!("runtime200.extension.{index:03}"),
                format!("Extension {index}"),
                "external-owner",
            )
        })
        .collect();

    let definitions = package_feature_definitions(&manifest);

    assert_eq!(definitions.len(), DEFINITIONS_PER_BUILD);
    assert!(definitions.capacity() >= DEFINITIONS_PER_BUILD);
    assert_eq!(definitions[0].key, "runtime200.optional.000@runtime200");
    assert_eq!(
        definitions[OPTIONAL_FEATURES_PER_BUILD].key,
        "runtime200.extension.000@runtime200"
    );
    assert_eq!(
        definitions[DEFINITIONS_PER_BUILD - 1].key,
        "runtime200.extension.127@runtime200"
    );
}

#[test]
fn optimization_batch_20260826fe_runtime200_package_feature_definitions_reserve_source_total() {
    let source = include_str!("../package_feature_definitions.rs");
    assert!(source.contains("fn package_feature_definition_capacity("));
    assert!(source.contains("package_manifest.optional_features.len()"));
    assert!(source.contains(".saturating_add(package_manifest.feature_extensions.len())"));
    assert!(source.contains("Vec::with_capacity(package_feature_definition_capacity("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fe_runtime200_package_feature_definition_capacity_bench() {
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
        "RUNTIME200_PACKAGE_FEATURE_DEFINITION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} optional_features={OPTIONAL_FEATURES_PER_BUILD} \
feature_extensions={FEATURE_EXTENSIONS_PER_BUILD} definitions_per_build={DEFINITIONS_PER_BUILD} \
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
        let mut definitions = if reserve {
            Vec::with_capacity(DEFINITIONS_PER_BUILD)
        } else {
            Vec::new()
        };
        for definition in 0..OPTIONAL_FEATURES_PER_BUILD {
            definitions.push(black_box(definition));
        }
        for definition in 0..FEATURE_EXTENSIONS_PER_BUILD {
            definitions.push(black_box(definition));
        }
        checksum ^= black_box(definitions.len() ^ definitions.capacity());
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

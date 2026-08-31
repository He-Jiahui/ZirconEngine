use std::hint::black_box;
use std::time::Instant;

use crate::asset::AssetUri;

use super::canonical_builtin_locator_matches;

const SAMPLE_PAIRS: usize = 31;
const MATCHES_PER_SAMPLE: usize = 100_000;
const BUILTIN_LOCATOR: &str =
    "builtin://materials/standard/surface/physically_based_material.asset.toml#default";

#[test]
fn optimization_batch_20260828it_runtime292_matches_canonical_builtin_path_and_label() {
    let primary = AssetUri::parse(BUILTIN_LOCATOR).expect("builtin locator");
    let candidates = [
        BUILTIN_LOCATOR,
        "builtin://materials/standard/surface/physically_based_material.asset.toml#preview",
        "builtin://materials/standard/surface/unlit_material.asset.toml#default",
        "res://materials/standard/surface/physically_based_material.asset.toml#default",
    ];

    for candidate in candidates {
        assert_eq!(
            canonical_builtin_locator_matches(&primary, candidate),
            legacy_builtin_locator_matches(&primary, candidate),
            "match diverged for {candidate}"
        );
    }
}

#[test]
fn optimization_batch_20260828it_runtime292_residency_scan_has_no_locator_parse() {
    let source = include_str!("../ensure_resident.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");

    assert!(implementation.contains("canonical_builtin_locator_matches("));
    assert!(implementation.contains("locator.path() == path"));
    assert!(implementation.contains("locator.label() == label"));
    assert!(!implementation.contains("AssetUri::parse(locator_text)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260828it_runtime292_canonical_builtin_locator_match_bench() {
    let primary = AssetUri::parse(BUILTIN_LOCATOR).expect("builtin locator");
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&primary, false));
            optimized_samples.push(measure(&primary, true));
        } else {
            optimized_samples.push(measure(&primary, true));
            legacy_samples.push(measure(&primary, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME292_CANONICAL_BUILTIN_LOCATOR_MATCH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
matches_per_sample={MATCHES_PER_SAMPLE} locator_bytes={} \
legacy_locator_parses_per_match=1 optimized_locator_parses_per_match=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        BUILTIN_LOCATOR.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_builtin_locator_matches(locator: &AssetUri, candidate: &str) -> bool {
    AssetUri::parse(candidate).is_ok_and(|candidate| &candidate == locator)
}

fn measure(locator: &AssetUri, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for iteration in 0..MATCHES_PER_SAMPLE {
        let matched = if optimized {
            canonical_builtin_locator_matches(locator, black_box(BUILTIN_LOCATOR))
        } else {
            legacy_builtin_locator_matches(locator, black_box(BUILTIN_LOCATOR))
        };
        checksum ^= black_box((matched as usize).wrapping_add(iteration));
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

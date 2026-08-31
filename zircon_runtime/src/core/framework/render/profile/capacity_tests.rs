use std::hint::black_box;
use std::time::Instant;

use super::{RenderProductProfile, RenderProfileBundle};

const SAMPLE_PAIRS: usize = 21;
const EXPANSIONS_PER_SAMPLE: usize = 8_192;
const PROFILES_PER_EXPANSION: usize = 5;

#[test]
fn optimization_batch_20260826gb_runtime223_profile_capacity_covers_owner_and_includes() {
    let profiles = RenderProfileBundle::default_render().required_profiles();

    assert_eq!(profiles.len(), PROFILES_PER_EXPANSION);
    assert!(profiles.capacity() >= PROFILES_PER_EXPANSION);
    assert_eq!(profiles[0], RenderProductProfile::DefaultRender);
    assert!(profiles.contains(&RenderProductProfile::CommonRenderApi));
    assert!(profiles.contains(&RenderProductProfile::Ui));
}

#[test]
fn optimization_batch_20260826gb_runtime223_profile_expansion_reserves_owner_and_includes() {
    let source = include_str!("../profile.rs");

    assert!(source.contains("self.includes.len().saturating_add(1)"));
    assert!(source.contains("Vec::with_capacity(profile_capacity)"));
    assert!(!source.contains("let mut profiles = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gb_runtime223_render_profile_expansion_capacity_bench() {
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
        "RUNTIME223_RENDER_PROFILE_EXPANSION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
expansions_per_sample={EXPANSIONS_PER_SAMPLE} profiles_per_expansion={PROFILES_PER_EXPANSION} \
legacy_preallocated_expansions=0 optimized_preallocated_expansions={EXPANSIONS_PER_SAMPLE} \
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
    for expansion in 0..EXPANSIONS_PER_SAMPLE {
        let mut profiles = if reserve {
            Vec::with_capacity(PROFILES_PER_EXPANSION)
        } else {
            Vec::new()
        };
        for profile in 0..PROFILES_PER_EXPANSION {
            profiles.push(black_box(expansion ^ profile));
        }
        checksum ^= black_box(profiles.len() ^ profiles.capacity());
        black_box(&profiles);
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

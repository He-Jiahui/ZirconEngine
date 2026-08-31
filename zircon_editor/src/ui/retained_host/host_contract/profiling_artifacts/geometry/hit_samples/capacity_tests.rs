use std::hint::black_box;
use std::time::Instant;

use super::{profile_hit_sample_capacity, PROFILE_HIT_SAMPLES_PER_FRAME};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 683;
const FRAMES_PER_BUILD: usize = 256;
const SAMPLES_PER_BUILD: usize = FRAMES_PER_BUILD * PROFILE_HIT_SAMPLES_PER_FRAME;

#[test]
fn optimization_batch_20260826fb_editor143_capacity_matches_three_samples_per_frame() {
    assert_eq!(PROFILE_HIT_SAMPLES_PER_FRAME, 3);
    assert_eq!(
        profile_hit_sample_capacity(FRAMES_PER_BUILD),
        SAMPLES_PER_BUILD
    );
    assert_eq!(profile_hit_sample_capacity(0), 0);
    assert_eq!(profile_hit_sample_capacity(usize::MAX), usize::MAX);
}

#[test]
fn optimization_batch_20260826fb_editor143_hit_samples_reserve_exact_frame_multiple() {
    let source = include_str!("../hit_samples.rs");
    assert!(source.contains("const PROFILE_HIT_SAMPLES_PER_FRAME: usize = 3;"));
    assert!(source.contains("Vec::with_capacity(profile_hit_sample_capacity(frames.len()))"));
    assert!(source.contains("Vec::with_capacity(PROFILE_HIT_SAMPLES_PER_FRAME)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fb_editor143_profile_hit_sample_capacity_bench() {
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
        "EDITOR143_PROFILE_HIT_SAMPLE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} frames_per_build={FRAMES_PER_BUILD} \
samples_per_build={SAMPLES_PER_BUILD} legacy_reservations_per_build=0 \
optimized_reservations_per_build=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut samples = if reserve {
            Vec::with_capacity(SAMPLES_PER_BUILD)
        } else {
            Vec::new()
        };
        for sample in 0..SAMPLES_PER_BUILD {
            samples.push(black_box(sample));
        }
        checksum ^= black_box(samples.len() ^ samples.capacity());
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

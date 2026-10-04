use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 120_000;
const TOKENS: [&str; 3] = [" RGBA8UNORM ", "rgba8UNORM_SRGB", "rgba8unorm-srgb"];

#[test]
fn borrowed_texture_format_contract_decoded_rgba8() {
    assert!(is_decoded_rgba8_format(TOKENS[0]));
    assert!(is_decoded_rgba8_format(TOKENS[1]));
    assert!(!is_decoded_rgba8_format(TOKENS[2]));
}

#[test]
#[ignore = "release performance gate"]
fn borrowed_texture_format_performance_release_decoded_rgba8() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
            (measure_legacy(), measure_borrowed())
        } else {
            let optimized_ns = measure_borrowed();
            (measure_legacy(), optimized_ns)
        };
        legacy_samples.push(legacy_ns);
        optimized_samples.push(optimized_ns);
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins07_decoded_rgba8_format_check sample_pairs={SAMPLE_PAIRS} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25 legacy_allocations_per_sample={} optimized_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
        csv(&legacy_samples),
        csv(&optimized_samples),
        LOOKUPS_PER_SAMPLE * TOKENS.len(),
    );
    assert!(
        improvement_percent >= 25,
        "borrowed decoded RGBA8 matching must improve P95 by at least 25%"
    );
}

fn measure_legacy() -> u128 {
    let started = Instant::now();
    let mut matched = 0_u64;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        for token in TOKENS {
            matched += u64::from(matches!(
                black_box(token).trim().to_ascii_lowercase().as_str(),
                RGBA8_UNORM_FORMAT | RGBA8_UNORM_SRGB_FORMAT
            ));
        }
    }
    black_box(matched);
    started.elapsed().as_nanos()
}

fn measure_borrowed() -> u128 {
    let started = Instant::now();
    let mut matched = 0_u64;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        for token in TOKENS {
            matched += u64::from(is_decoded_rgba8_format(black_box(token)));
        }
    }
    black_box(matched);
    started.elapsed().as_nanos()
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

use std::hint::black_box;
use std::time::Instant;

use super::{canonical_discrete_name_matches, AudioChannelLayout};

const SAMPLE_PAIRS: usize = 31;
const CHECKS_PER_SAMPLE: usize = 200_000;

#[test]
fn optimization_batch_20260829y_runtime298_discrete_name_check_preserves_contract() {
    for channel_count in [1, 16, u16::MAX] {
        assert!(AudioChannelLayout::discrete(channel_count).is_canonical_discrete_layout());
    }

    for (name, channel_count) in [
        ("discrete_01", 1),
        ("discrete_65536", u16::MAX),
        ("discrete_-1", 1),
        ("discrete_1x", 1),
        ("discrete_2", 1),
        ("discrete_", 1),
    ] {
        assert!(!canonical_discrete_name_matches(name, channel_count));
    }
}

#[test]
fn optimization_batch_20260829y_runtime298_discrete_name_check_avoids_formatting() {
    let source = include_str!("../channel_layout.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let body = implementation
        .split("pub fn is_canonical_discrete_layout")
        .nth(1)
        .and_then(|body| body.split("pub fn is_valid_contract_layout").next())
        .expect("discrete layout validator");

    assert!(body.contains("canonical_discrete_name_matches"));
    assert!(!body.contains("format!("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829y_runtime298_allocation_free_discrete_channel_name_check_bench() {
    let cases = [
        ("discrete_1", 1),
        ("discrete_16", 16),
        ("discrete_65535", u16::MAX),
        ("discrete_01", 1),
        ("discrete_65536", u16::MAX),
        ("discrete_4095x", 4095),
        ("custom_layout", 12),
    ];
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, &cases));
            optimized_samples.push(measure(true, &cases));
        } else {
            optimized_samples.push(measure(true, &cases));
            legacy_samples.push(measure(false, &cases));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME298_ALLOCATION_FREE_DISCRETE_CHANNEL_NAME_CHECK_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} case_count={} \
legacy_result_allocations_per_check=1 optimized_result_allocations_per_check=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        cases.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_discrete_name_matches(name: &str, channel_count: u16) -> bool {
    name == format!("discrete_{channel_count}")
}

fn measure(optimized: bool, cases: &[(&str, u16)]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for index in 0..CHECKS_PER_SAMPLE {
        let (name, channel_count) = black_box(cases[index % cases.len()]);
        let matches = if optimized {
            canonical_discrete_name_matches(name, channel_count)
        } else {
            legacy_discrete_name_matches(name, channel_count)
        };
        checksum = checksum.wrapping_add(usize::from(black_box(matches)));
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

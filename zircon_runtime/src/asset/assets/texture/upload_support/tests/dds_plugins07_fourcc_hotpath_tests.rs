use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 120_000;

#[test]
fn borrowed_texture_format_contract_dds_fourcc() {
    let bytes = dds_bytes(*b"DxT1");
    assert!(dds_fourcc_matches(&bytes, "dxt1"));
    assert!(dds_fourcc_matches(&bytes, "DXT1"));
    assert!(!dds_fourcc_matches(&bytes, "dxt5"));
}

#[test]
#[ignore = "release performance gate"]
fn borrowed_texture_format_performance_release_dds_fourcc() {
    let bytes = dds_bytes(*b"DxT1");
    let expected = ["dxt1", "dxt1", "dxt5"];
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
            (
                measure_legacy(&bytes, &expected),
                measure_borrowed(&bytes, &expected),
            )
        } else {
            let optimized_ns = measure_borrowed(&bytes, &expected);
            (measure_legacy(&bytes, &expected), optimized_ns)
        };
        legacy_samples.push(legacy_ns);
        optimized_samples.push(optimized_ns);
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins07_dds_fourcc_validation sample_pairs={SAMPLE_PAIRS} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25 legacy_allocations_per_sample={} optimized_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
        csv(&legacy_samples),
        csv(&optimized_samples),
        LOOKUPS_PER_SAMPLE * expected.len(),
    );
    assert!(
        improvement_percent >= 25,
        "borrowed DDS FourCC validation must improve P95 by at least 25%"
    );
}

fn dds_bytes(fourcc: [u8; 4]) -> [u8; 88] {
    let mut bytes = [0_u8; 88];
    bytes[84..88].copy_from_slice(&fourcc);
    bytes
}

fn measure_legacy(bytes: &[u8], expected: &[&str]) -> u128 {
    let started = Instant::now();
    let mut matched = 0_u64;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        for candidate in expected {
            matched += u64::from(
                dds_fourcc(black_box(bytes))
                    .is_some_and(|value| value.to_ascii_lowercase() == black_box(*candidate)),
            );
        }
    }
    black_box(matched);
    started.elapsed().as_nanos()
}

fn measure_borrowed(bytes: &[u8], expected: &[&str]) -> u128 {
    let started = Instant::now();
    let mut matched = 0_u64;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        for candidate in expected {
            matched += u64::from(dds_fourcc_matches(black_box(bytes), black_box(*candidate)));
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

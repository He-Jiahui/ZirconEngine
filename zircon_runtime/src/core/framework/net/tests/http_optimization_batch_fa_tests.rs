use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const VALUES_PER_SAMPLE: usize = 262_144;

#[test]
fn optimization_batch_fa_runtime459_preserves_http_byte_range_values() {
    for (start, end_inclusive) in [
        (0, 0),
        (1, 99),
        (4_096, 8_191),
        (u32::MAX as u64, u64::MAX),
        (u64::MAX, u64::MAX),
    ] {
        assert_eq!(
            byte_range_header_value(start, end_inclusive),
            format!("bytes={start}-{end_inclusive}")
        );
    }

    let request = NetHttpRequestDescriptor::new(
        NetRequestId::new(7),
        NetHttpMethod::Get,
        "https://example.invalid/chunk",
    )
    .with_header("accept", "application/octet-stream")
    .with_header("Range", "bytes=1-2")
    .with_byte_range(4_096, 8_191);
    assert_eq!(
        request.headers,
        vec![
            ("accept".to_string(), "application/octet-stream".to_string()),
            ("range".to_string(), "bytes=4096-8191".to_string()),
        ]
    );
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fa_runtime459_direct_http_byte_range_benchmark() {
    for _ in 0..4 {
        black_box(measure_legacy());
        black_box(measure_optimized());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    report_performance(&legacy_samples, &optimized_samples);
}

fn measure_legacy() -> u128 {
    measure(|start, end_inclusive| format!("bytes={start}-{end_inclusive}"))
}

fn measure_optimized() -> u128 {
    measure(byte_range_header_value)
}

fn measure(mut encode: impl FnMut(u64, u64) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for index in 0..VALUES_PER_SAMPLE {
        let start = black_box((index as u64).wrapping_mul(1_048_583));
        let end_inclusive = black_box(start.saturating_add(65_535));
        let value = encode(start, end_inclusive);
        checksum = checksum.wrapping_add(black_box(value.len()));
        black_box(value);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
    let legacy_p95 = nearest_rank_p95(legacy_samples);
    let optimized_p95 = nearest_rank_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "RUNTIME459_DIRECT_HTTP_BYTE_RANGE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} values_per_sample={VALUES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=30",
        csv(legacy_samples),
        csv(optimized_samples),
    );
    assert!(
        optimized_p95 <= legacy_p95.saturating_mul(70) / 100,
        "direct HTTP byte-range construction must reduce P95 by at least 30%"
    );
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

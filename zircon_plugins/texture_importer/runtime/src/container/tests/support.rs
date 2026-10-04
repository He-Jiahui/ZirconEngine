use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::format_parse_error_message;

const SAMPLE_PAIRS: usize = 21;
const FORMAT_CALLS_PER_SAMPLE: usize = 8_192;

#[test]
fn parse_error_message_preserves_borrowed_detail_text() {
    let message = format_parse_error_message(
        Path::new("sample.ktx2"),
        format_args!("level {} requires at least {} bytes, got {}", 7, 64, 11),
    );

    assert_eq!(
        message,
        "parse texture container sample.ktx2: level 7 requires at least 64 bytes, got 11"
    );
}

#[test]
#[ignore = "release-only performance contract"]
fn benchmark_single_allocation_parse_error_formatting() {
    let source_path = PathBuf::from("fixtures/oversized_texture.ktx2");
    let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);

    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_raw.push(measure_legacy(&source_path));
            optimized_raw.push(measure_optimized(&source_path));
        } else {
            optimized_raw.push(measure_optimized(&source_path));
            legacy_raw.push(measure_legacy(&source_path));
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
    let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
    let improvement_percent = legacy_p95_ns
        .saturating_sub(optimized_p95_ns)
        .saturating_mul(100)
        / legacy_p95_ns.max(1);
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85),
        "single-allocation formatting must improve P95 by at least 15%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "PERF_RESULT task=plugins07_container_parse_error_formatting sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank calls_per_sample={FORMAT_CALLS_PER_SAMPLE} legacy_allocations_per_call=2 optimized_allocations_per_call=1 legacy_detail_allocations_per_sample={FORMAT_CALLS_PER_SAMPLE} optimized_detail_allocations_per_sample=0 threshold_percent=15 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
        raw_samples(&legacy_raw),
        raw_samples(&optimized_raw)
    );
}

fn measure_legacy(source_path: &Path) -> u64 {
    let started = Instant::now();
    for index in 0..FORMAT_CALLS_PER_SAMPLE {
        let detail = format!(
            "level {} requires at least {} bytes, got {}",
            index % 12,
            64 + index % 128,
            index % 63
        );
        black_box(format!(
            "parse texture container {}: {}",
            source_path.display(),
            detail
        ));
    }
    nanos(started)
}

fn measure_optimized(source_path: &Path) -> u64 {
    let started = Instant::now();
    for index in 0..FORMAT_CALLS_PER_SAMPLE {
        black_box(format_parse_error_message(
            source_path,
            format_args!(
                "level {} requires at least {} bytes, got {}",
                index % 12,
                64 + index % 128,
                index % 63
            ),
        ));
    }
    nanos(started)
}

fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw_samples(samples: &[u64]) -> String {
    let values = samples
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}

fn nanos(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

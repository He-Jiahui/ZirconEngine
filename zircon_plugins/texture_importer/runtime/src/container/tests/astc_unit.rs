use std::hint::black_box;
use std::time::Instant;

use super::*;

const BENCHMARK_FORMATS: usize = 65_536;
const BENCHMARK_SAMPLE_PAIRS: usize = 21;
const BENCHMARK_THRESHOLD_PERCENT: u128 = 25;

fn legacy_canonical_format(block_x: u8, block_y: u8, block_z: u8) -> String {
    assert!(supported_block_format(block_x, block_y, block_z).is_some());
    format!("astc/{block_x}x{block_y}x{block_z}")
}

fn optimized_canonical_format(block_x: u8, block_y: u8, block_z: u8) -> String {
    supported_block_format(block_x, block_y, block_z)
        .unwrap()
        .to_owned()
}

fn measure_format_creation(mut create: impl FnMut() -> String) -> u128 {
    let timer = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..BENCHMARK_FORMATS {
        checksum += black_box(create()).len();
    }
    black_box(checksum);
    timer.elapsed().as_nanos()
}

fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95 - 1) / 100]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn run_format_benchmark(marker: &str, block_x: u8, block_y: u8, block_z: u8) {
    assert_eq!(
        legacy_canonical_format(block_x, block_y, block_z),
        optimized_canonical_format(block_x, block_y, block_z)
    );
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    for sample_index in 0..BENCHMARK_SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_format_creation(|| {
                legacy_canonical_format(
                    black_box(block_x),
                    black_box(block_y),
                    black_box(block_z),
                )
            }));
            optimized_samples.push(measure_format_creation(|| {
                optimized_canonical_format(
                    black_box(block_x),
                    black_box(block_y),
                    black_box(block_z),
                )
            }));
        } else {
            optimized_samples.push(measure_format_creation(|| {
                optimized_canonical_format(
                    black_box(block_x),
                    black_box(block_y),
                    black_box(block_z),
                )
            }));
            legacy_samples.push(measure_format_creation(|| {
                legacy_canonical_format(
                    black_box(block_x),
                    black_box(block_y),
                    black_box(block_z),
                )
            }));
        }
    }

    let legacy_raw = legacy_samples.clone();
    let optimized_raw = optimized_samples.clone();
    let legacy_p95_ns = nearest_rank_p95(&mut legacy_samples);
    let optimized_p95_ns = nearest_rank_p95(&mut optimized_samples);
    let improvement_percent = legacy_p95_ns
        .saturating_sub(optimized_p95_ns)
        .saturating_mul(100)
        / legacy_p95_ns.max(1);

    println!(
        "PERF_RESULT {marker} formats_per_sample={} sample_pairs={} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_integer_format_calls_per_sample={} optimized_integer_format_calls_per_sample=0 legacy_p95_ns={} optimized_p95_ns={} improvement_percent={} threshold_percent={} legacy_ns={} optimized_ns={}",
        BENCHMARK_FORMATS,
        BENCHMARK_SAMPLE_PAIRS,
        BENCHMARK_FORMATS,
        legacy_p95_ns,
        optimized_p95_ns,
        improvement_percent,
        BENCHMARK_THRESHOLD_PERCENT,
        sample_csv(&legacy_raw),
        sample_csv(&optimized_raw),
    );

    assert_eq!(BENCHMARK_SAMPLE_PAIRS, legacy_raw.len());
    assert_eq!(BENCHMARK_SAMPLE_PAIRS, optimized_raw.len());
    assert!(
        improvement_percent >= BENCHMARK_THRESHOLD_PERCENT,
        "{marker} P95 improvement {improvement_percent}% misses {BENCHMARK_THRESHOLD_PERCENT}% gate"
    );
}

#[test]
fn canonical_astc_format_maps_supported_2d_footprint() {
    assert_eq!(supported_block_format(12, 10, 1), Some("astc/12x10x1"));
}

#[test]
fn canonical_astc_format_maps_supported_3d_footprint() {
    assert_eq!(supported_block_format(6, 6, 6), Some("astc/6x6x6"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn benchmark_canonical_astc_format_2d() {
    run_format_benchmark("plugins07_canonical_astc_format_2d", 12, 10, 1);
}

#[test]
#[ignore = "release-only performance evidence"]
fn benchmark_canonical_astc_format_3d() {
    run_format_benchmark("plugins07_canonical_astc_format_3d", 6, 6, 6);
}

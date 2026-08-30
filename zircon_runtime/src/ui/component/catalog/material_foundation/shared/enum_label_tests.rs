use std::hint::black_box;
use std::time::Instant;

use super::enum_label;

const SAMPLE_PAIRS: usize = 21;
const LABELS_PER_SAMPLE: usize = 131_072;
const FIXTURE: &str = "filled_tonal_surface_container_high_emphasis_action";

#[test]
fn optimization_batch_20260826di_runtime152_enum_label_preserves_segment_contract() {
    assert_eq!(enum_label("filled_tonal"), "Filled Tonal");
    assert_eq!(enum_label("a__b"), "A  B");
    assert_eq!(enum_label("_leading"), " Leading");
    assert_eq!(enum_label("trailing_"), "Trailing ");
    assert_eq!(enum_label("élan_mode"), "élan Mode");
}

#[test]
fn optimization_batch_20260826di_runtime152_enum_label_uses_one_output_buffer() {
    let label = enum_label(FIXTURE);
    assert_eq!(label.len(), label.capacity());

    let source = include_str!("../shared.rs");
    assert!(source.contains("let mut label = String::with_capacity(option.len());"));
    assert!(source.contains("label.push(first.to_ascii_uppercase());"));
    assert!(!source.contains("format!(\"{}{}\", first.to_ascii_uppercase()"));
    assert!(!source.contains(".collect::<Vec<_>>()\n        .join(\" \")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826di_runtime152_enum_label_single_buffer_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_enum_label));
            optimized_samples.push(measure(enum_label));
        } else {
            optimized_samples.push(measure(enum_label));
            legacy_samples.push(measure(legacy_enum_label));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME152_ENUM_LABEL_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
labels_per_sample={LABELS_PER_SAMPLE} segments_per_label=7 legacy_minimum_allocations_per_label=9 \
optimized_allocations_per_label=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single-buffer enum label P95 {optimized_p95_ns}ns must be at most 70% of segmented collector P95 {legacy_p95_ns}ns"
    );
}

fn legacy_enum_label(option: &str) -> String {
    option
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn measure(render: fn(&str) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LABELS_PER_SAMPLE {
        checksum ^= black_box(render(black_box(FIXTURE))).len();
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

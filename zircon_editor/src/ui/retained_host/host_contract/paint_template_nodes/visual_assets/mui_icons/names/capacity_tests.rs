use std::hint::black_box;
use std::time::Instant;

use super::pascal_case_ligature_name;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 512;
const NAME_BYTES: usize = 4_096;
const NAME_PARTS: usize = 256;

#[test]
fn optimization_batch_20260826fm_editor154_capacity_preserves_ligature_name_projection() {
    let name = (0..NAME_PARTS)
        .map(|index| format!("icon_{index:03}_"))
        .collect::<String>();

    let projected = pascal_case_ligature_name(&name).expect("ligature name should be valid");

    assert!(projected.starts_with("Icon000Icon001Icon002"));
    assert!(projected.ends_with("Icon255"));
    assert!(projected.capacity() >= name.len());
    assert!(projected
        .chars()
        .all(|character| character.is_ascii_alphanumeric()));
}

#[test]
fn optimization_batch_20260826fm_editor154_ligature_output_reserves_input_bytes() {
    let source = include_str!("../names.rs");
    assert!(source.contains("let mut out = String::with_capacity(name.len());"));
    assert!(!source.contains("let mut out = String::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fm_editor154_mui_icon_name_capacity_bench() {
    let name = "a".repeat(NAME_BYTES);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&name, false));
            optimized_samples.push(measure(&name, true));
        } else {
            optimized_samples.push(measure(&name, true));
            legacy_samples.push(measure(&name, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR154_MUI_ICON_NAME_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} name_bytes={NAME_BYTES} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(name: &str, reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut output = if reserve {
            String::with_capacity(name.len())
        } else {
            String::new()
        };
        for character in black_box(name).chars() {
            output.push(character);
        }
        checksum ^= black_box(output.len() ^ output.capacity());
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

use std::hint::black_box;
use std::time::Instant;

use super::preset_staging_file_name;

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 150_000;

#[test]
fn optimization_batch_20260829ah_editor253_staging_names_preserve_exact_text() {
    assert_eq!(
        preset_staging_file_name("desktop.zpreset", 42, 9_876_543),
        ".desktop.zpreset.42-9876543.staging"
    );
    assert_eq!(
        preset_staging_file_name("preset.zpreset", u32::MAX, u64::MAX),
        format!(".preset.zpreset.{}-{}.staging", u32::MAX, u64::MAX)
    );
}

#[test]
fn optimization_batch_20260829ah_editor253_transaction_builds_one_name_buffer() {
    let source = include_str!("../preset.rs");
    let constructor = source
        .split("impl PresetWriteTransaction")
        .nth(1)
        .expect("preset write transaction implementation")
        .split("fn write_and_commit")
        .next()
        .expect("preset write transaction constructor");

    assert!(constructor.contains("preset_staging_file_name("));
    assert!(!constructor.contains("let nonce = format!"));
    assert!(!constructor.contains(".to_owned()"));
    assert!(!constructor.contains("format!("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ah_editor253_single_buffer_export_preset_staging_name_bench() {
    let file_name = "shipping_desktop_with_a_long_configuration_name.zpreset";
    let process_id = 4_294_967_000;
    let nonce = 18_446_744_073_709_000_000;
    assert_eq!(
        preset_staging_file_name(file_name, process_id, nonce),
        legacy_preset_staging_file_name(file_name, process_id, nonce)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, file_name, process_id, nonce));
            optimized_samples.push(measure(true, file_name, process_id, nonce));
        } else {
            optimized_samples.push(measure(true, file_name, process_id, nonce));
            legacy_samples.push(measure(false, file_name, process_id, nonce));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR253_SINGLE_BUFFER_EXPORT_PRESET_STAGING_NAME_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} legacy_string_allocations_per_build=3 \
optimized_string_allocations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_preset_staging_file_name(file_name: &str, process_id: u32, nonce: u64) -> String {
    let nonce = format!("{process_id}-{nonce}");
    let file_name = file_name.to_owned();
    format!(".{file_name}.{nonce}.staging")
}

fn measure(optimized: bool, file_name: &str, process_id: u32, nonce: u64) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let staging_name = if optimized {
            preset_staging_file_name(black_box(file_name), process_id, nonce)
        } else {
            legacy_preset_staging_file_name(black_box(file_name), process_id, nonce)
        };
        checksum = checksum.wrapping_add(black_box(staging_name).len());
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

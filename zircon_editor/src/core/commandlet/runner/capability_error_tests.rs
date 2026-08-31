use std::hint::black_box;
use std::time::Instant;

use super::format_missing_capabilities;

const SAMPLE_PAIRS: usize = 21;
const FORMATS_PER_SAMPLE: usize = 65_536;

#[test]
fn optimization_batch_20260826dq_editor106_commandlet_capabilities_preserve_message() {
    let capabilities = vec![
        "asset.migrate".to_string(),
        "project.write".to_string(),
        "plugin.inspect".to_string(),
    ];
    assert_eq!(
        format_missing_capabilities(&capabilities),
        "commandlet requires unavailable capabilities: asset.migrate, project.write, plugin.inspect"
    );
    assert_eq!(
        format_missing_capabilities(&[]),
        "commandlet requires unavailable capabilities: "
    );
}

#[test]
fn optimization_batch_20260826dq_editor106_commandlet_capabilities_write_one_exact_buffer() {
    let capabilities = fixture_capabilities();
    let message = format_missing_capabilities(&capabilities);
    assert_eq!(message.capacity(), message.len());

    let source = include_str!("../runner.rs");
    assert!(source.contains("error: Some(format_missing_capabilities(&capabilities))"));
    assert!(source.contains("let capability_bytes = capabilities.iter().map(String::len)"));
    assert!(source.contains("let mut message = String::with_capacity("));
    assert!(!source.contains("capabilities.join(\", \")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dq_editor106_commandlet_capability_single_buffer_bench() {
    let capabilities = fixture_capabilities();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&capabilities, legacy_format));
            optimized_samples.push(measure(&capabilities, format_missing_capabilities));
        } else {
            optimized_samples.push(measure(&capabilities, format_missing_capabilities));
            legacy_samples.push(measure(&capabilities, legacy_format));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR106_COMMANDLET_CAPABILITY_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
formats_per_sample={FORMATS_PER_SAMPLE} legacy_allocations_per_format=2 \
optimized_allocations_per_format=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single-buffer commandlet capability formatting P95 {optimized_p95_ns}ns must be at most 70% of join formatting P95 {legacy_p95_ns}ns"
    );
}

fn fixture_capabilities() -> Vec<String> {
    (0..16)
        .map(|index| format!("editor.commandlet.capability.production_{index:02}"))
        .collect()
}

fn legacy_format(capabilities: &[String]) -> String {
    format!(
        "commandlet requires unavailable capabilities: {}",
        capabilities.join(", ")
    )
}

fn measure(capabilities: &[String], render: fn(&[String]) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..FORMATS_PER_SAMPLE {
        checksum ^= black_box(render(black_box(capabilities))).len();
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

use std::hint::black_box;
use std::time::Instant;

use super::native_plugin_discovery_hint;

const SAMPLE_PAIRS: usize = 21;
const HINTS_PER_SAMPLE: usize = 16_384;
const PLUGINS_PER_HINT: usize = 32;

#[test]
fn optimization_batch_20260826dn_runtime157_native_plugin_discovery_hint_preserves_output() {
    let ids = ["render", "physics", "editor_tools"];
    assert_eq!(
        native_plugin_discovery_hint(ids.into_iter()),
        "discovered native plugins: render, physics, editor_tools"
    );
    assert_eq!(
        native_plugin_discovery_hint(std::iter::empty()),
        "no native plugin manifests were discovered"
    );
}

#[test]
fn optimization_batch_20260826dn_runtime157_native_plugin_discovery_hint_uses_exact_buffer() {
    let ids = fixture_plugin_ids();
    let hint = optimized_hint(&ids);
    assert_eq!(hint.len(), hint.capacity());

    let source = include_str!("../lifecycle.rs");
    assert!(source.contains("let mut hint = String::with_capacity(capacity);"));
    assert!(source.contains("hint.push_str(plugin_id);"));
    assert!(!source.contains("let discovered = report"));
    assert!(!source.contains("format!(\"discovered native plugins: {discovered}\")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dn_runtime157_native_plugin_discovery_hint_single_buffer_bench() {
    let ids = fixture_plugin_ids();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&ids, legacy_hint));
            optimized_samples.push(measure(&ids, optimized_hint));
        } else {
            optimized_samples.push(measure(&ids, optimized_hint));
            legacy_samples.push(measure(&ids, legacy_hint));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME157_NATIVE_PLUGIN_DISCOVERY_HINT_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
hints_per_sample={HINTS_PER_SAMPLE} plugins_per_hint={PLUGINS_PER_HINT} \
legacy_intermediate_allocations_per_hint=2 optimized_intermediate_allocations_per_hint=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single-buffer discovery hint P95 {optimized_p95_ns}ns must be at most 70% of collected plugin-id formatting P95 {legacy_p95_ns}ns"
    );
}

fn fixture_plugin_ids() -> Vec<String> {
    (0..PLUGINS_PER_HINT)
        .map(|index| format!("native_plugin_{index:02}"))
        .collect()
}

fn optimized_hint(plugin_ids: &[String]) -> String {
    native_plugin_discovery_hint(plugin_ids.iter().map(String::as_str))
}

fn legacy_hint(plugin_ids: &[String]) -> String {
    let discovered = plugin_ids
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    if discovered.is_empty() {
        "no native plugin manifests were discovered".to_string()
    } else {
        format!("discovered native plugins: {discovered}")
    }
}

fn measure(plugin_ids: &[String], render: fn(&[String]) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..HINTS_PER_SAMPLE {
        checksum ^= black_box(render(black_box(plugin_ids))).len();
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

use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::platform::RuntimeTargetMode;

use super::target_modes_status_label;

const SAMPLE_PAIRS: usize = 21;
const CALLS_PER_SAMPLE: usize = 262_144;
const MODES: [RuntimeTargetMode; 3] = [
    RuntimeTargetMode::ClientRuntime,
    RuntimeTargetMode::ServerRuntime,
    RuntimeTargetMode::EditorHost,
];

#[test]
fn optimization_batch_20260826dh_editor97_target_mode_join_preserves_status_labels() {
    assert_eq!(target_modes_status_label(&[]), "all");
    assert_eq!(target_modes_status_label(&MODES), "client, server, editor");
    assert_eq!(
        target_modes_status_label(&[
            RuntimeTargetMode::EditorHost,
            RuntimeTargetMode::ClientRuntime,
        ]),
        "editor, client"
    );
}

#[test]
fn optimization_batch_20260826dh_editor97_target_mode_join_uses_one_result_buffer() {
    let label = target_modes_status_label(&MODES);
    assert_eq!(label.len(), label.capacity());

    let source = include_str!("../status.rs");
    assert!(source.contains("String::with_capacity(capacity)"));
    assert!(source.contains("label.push_str(target_mode_status_name(*mode))"));
    assert!(!source.contains("collect::<Vec<_>>()"));
    assert!(!source.contains(".join(\", \")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dh_editor97_target_mode_direct_join_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_target_modes_status_label));
            optimized_samples.push(measure(target_modes_status_label));
        } else {
            optimized_samples.push(measure(target_modes_status_label));
            legacy_samples.push(measure(legacy_target_modes_status_label));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR97_TARGET_MODE_DIRECT_JOIN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
calls_per_sample={CALLS_PER_SAMPLE} modes_per_call={} \
legacy_temporary_vec_allocations_per_sample={CALLS_PER_SAMPLE} \
optimized_temporary_vec_allocations_per_sample=0 result_allocations_per_sample={CALLS_PER_SAMPLE} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        MODES.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "direct target-mode join P95 {optimized_p95_ns}ns must be at most 70% of temporary-Vec join P95 {legacy_p95_ns}ns"
    );
}

fn legacy_target_modes_status_label(target_modes: &[RuntimeTargetMode]) -> String {
    if target_modes.is_empty() {
        return "all".to_string();
    }
    target_modes
        .iter()
        .map(|mode| match mode {
            RuntimeTargetMode::ClientRuntime => "client",
            RuntimeTargetMode::ServerRuntime => "server",
            RuntimeTargetMode::EditorHost => "editor",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn measure(render: fn(&[RuntimeTargetMode]) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..CALLS_PER_SAMPLE {
        checksum ^= black_box(render(black_box(&MODES))).len();
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

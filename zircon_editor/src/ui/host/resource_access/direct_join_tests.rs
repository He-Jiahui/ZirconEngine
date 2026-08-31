use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::resource::ResourceDiagnostic;

use super::render_diagnostics;

const SAMPLE_PAIRS: usize = 21;
const RENDERS_PER_SAMPLE: usize = 16_384;
const DIAGNOSTICS_PER_RENDER: usize = 32;

#[test]
fn optimization_batch_20260826dm_editor102_resource_diagnostic_preserves_order_and_empty() {
    let diagnostics = [
        ResourceDiagnostic::error("shader compile failed"),
        ResourceDiagnostic::error("fallback material unavailable"),
    ];
    assert_eq!(
        render_diagnostics(&diagnostics),
        "shader compile failed; fallback material unavailable"
    );
    assert_eq!(render_diagnostics(&[]), "");
}

#[test]
fn optimization_batch_20260826dm_editor102_resource_diagnostic_uses_exact_output_buffer() {
    let diagnostics = fixture_diagnostics();
    let rendered = render_diagnostics(&diagnostics);
    assert_eq!(rendered.len(), rendered.capacity());

    let source = include_str!("../resource_access.rs");
    assert!(source.contains("let mut rendered = String::with_capacity(capacity);"));
    assert!(source.contains("rendered.push_str(&diagnostic.message);"));
    assert!(!source.contains("collect::<Vec<_>>()"));
    assert!(!source.contains(".join(\"; \")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dm_editor102_resource_diagnostic_direct_join_bench() {
    let diagnostics = fixture_diagnostics();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&diagnostics, legacy_render_diagnostics));
            optimized_samples.push(measure(&diagnostics, render_diagnostics));
        } else {
            optimized_samples.push(measure(&diagnostics, render_diagnostics));
            legacy_samples.push(measure(&diagnostics, legacy_render_diagnostics));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR102_RESOURCE_DIAGNOSTIC_DIRECT_JOIN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
renders_per_sample={RENDERS_PER_SAMPLE} diagnostics_per_render={DIAGNOSTICS_PER_RENDER} \
legacy_temporary_vecs_per_sample={RENDERS_PER_SAMPLE} optimized_temporary_vecs_per_sample=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "direct resource diagnostic joining P95 {optimized_p95_ns}ns must be at most 70% of message-vector joining P95 {legacy_p95_ns}ns"
    );
}

fn fixture_diagnostics() -> Vec<ResourceDiagnostic> {
    (0..DIAGNOSTICS_PER_RENDER)
        .map(|index| ResourceDiagnostic::error(format!("resource diagnostic {index:02}")))
        .collect()
}

fn legacy_render_diagnostics(diagnostics: &[ResourceDiagnostic]) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>()
        .join("; ")
}

fn measure(
    diagnostics: &[ResourceDiagnostic],
    render: fn(&[ResourceDiagnostic]) -> String,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..RENDERS_PER_SAMPLE {
        checksum ^= black_box(render(black_box(diagnostics))).len();
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

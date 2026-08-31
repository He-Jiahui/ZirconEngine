use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828hz_runtime_reference_parts_reuse_payload_allocations() {
    let payload = benchmark_payload(4 * 1024, 2 * 1024);
    let reference_allocation = payload.reference.as_ptr();
    let source_allocation = payload.source.as_ref().unwrap().source_surface.as_ptr();
    let locator_allocation = payload
        .source
        .as_ref()
        .unwrap()
        .locator
        .as_ref()
        .unwrap()
        .as_ptr();

    let (kind, reference, source) = into_reference_parts(payload);
    let source = source.unwrap();

    assert_eq!(kind, UiDragPayloadKind::Asset);
    assert_eq!(reference.as_ptr(), reference_allocation);
    assert_eq!(source.source_surface.as_ptr(), source_allocation);
    assert_eq!(
        source.locator.as_ref().unwrap().as_ptr(),
        locator_allocation
    );
}

#[test]
fn optimization_batch_20260828hz_runtime_drop_reference_moves_owned_source_metadata() {
    let source = include_str!("../reference.rs");
    let drop_reference = source
        .split("pub(super) fn drop_reference")
        .nth(1)
        .and_then(|body| body.split("fn into_reference_parts").next())
        .expect("drop reference implementation");

    assert!(drop_reference.contains("into_reference_parts(payload)"));
    assert!(!drop_reference.contains("payload.source.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hz_runtime_owned_drag_reference_payload_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 128;

    black_box(legacy_reference_parts(benchmark_payload(
        8 * 1024,
        8 * 1024,
    )));
    black_box(into_reference_parts(benchmark_payload(8 * 1024, 8 * 1024)));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_inputs = (0..ITERATIONS)
            .map(|_| benchmark_payload(8 * 1024, 8 * 1024))
            .collect::<Vec<_>>();
        let optimized_inputs = (0..ITERATIONS)
            .map(|_| benchmark_payload(8 * 1024, 8 * 1024))
            .collect::<Vec<_>>();
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_payloads(legacy_inputs, legacy_reference_parts));
            optimized_samples.push(measure_payloads(optimized_inputs, into_reference_parts));
        } else {
            optimized_samples.push(measure_payloads(optimized_inputs, into_reference_parts));
            legacy_samples.push(measure_payloads(legacy_inputs, legacy_reference_parts));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME272_OWNED_DRAG_REFERENCE_PAYLOAD_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_payload(reference_bytes: usize, field_bytes: usize) -> UiDragPayload {
    let field = "x".repeat(field_bytes);
    UiDragPayload::new(UiDragPayloadKind::Asset, "r".repeat(reference_bytes)).with_source(
        UiDragSourceMetadata {
            source_surface: field.clone(),
            source_control_id: field.clone(),
            asset_uuid: Some(field.clone()),
            locator: Some(field.clone()),
            display_name: Some(field.clone()),
            asset_kind: Some(field.clone()),
            extension: Some(field),
        },
    )
}

fn legacy_reference_parts(
    payload: UiDragPayload,
) -> (UiDragPayloadKind, String, Option<UiDragSourceMetadata>) {
    let source = payload.source.clone();
    (payload.kind, payload.reference, source)
}

fn measure_payloads(
    payloads: Vec<UiDragPayload>,
    mut split: impl FnMut(UiDragPayload) -> (UiDragPayloadKind, String, Option<UiDragSourceMetadata>),
) -> u128 {
    let started = Instant::now();
    for payload in payloads {
        black_box(split(black_box(payload)));
    }
    started.elapsed().as_nanos()
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

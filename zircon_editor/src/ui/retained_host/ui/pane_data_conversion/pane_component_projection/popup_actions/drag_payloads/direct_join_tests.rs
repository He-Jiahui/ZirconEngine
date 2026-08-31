use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::UiDragPayloadKind;

use super::join_drag_payloads;

const SAMPLE_PAIRS: usize = 21;
const CALLS_PER_SAMPLE: usize = 262_144;
const PAYLOADS: [UiDragPayloadKind; 3] = [
    UiDragPayloadKind::Asset,
    UiDragPayloadKind::SceneInstance,
    UiDragPayloadKind::Object,
];

#[test]
fn optimization_batch_20260826dd_editor93_drag_payload_join_preserves_protocol_order() {
    assert_eq!(join_drag_payloads(&[]), "");
    assert_eq!(join_drag_payloads(&PAYLOADS), "asset,scene-instance,object");
    assert_eq!(
        join_drag_payloads(&[
            UiDragPayloadKind::Object,
            UiDragPayloadKind::Asset,
            UiDragPayloadKind::Object,
        ]),
        "object,asset,object"
    );
}

#[test]
fn optimization_batch_20260826dd_editor93_drag_payload_join_uses_one_result_buffer() {
    let source = include_str!("../drag_payloads.rs");

    assert!(source.contains("String::with_capacity(capacity)"));
    assert!(source.contains("joined.push_str(kind.as_str())"));
    assert!(!source.contains("collect::<Vec<_>>()"));
    assert!(!source.contains(".join(\",\")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dd_editor93_drag_payload_direct_join_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_join_drag_payloads));
            optimized_samples.push(measure(join_drag_payloads));
        } else {
            optimized_samples.push(measure(join_drag_payloads));
            legacy_samples.push(measure(legacy_join_drag_payloads));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR93_DRAG_PAYLOAD_DIRECT_JOIN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
calls_per_sample={CALLS_PER_SAMPLE} payloads_per_call={} \
legacy_temporary_vec_allocations_per_sample={CALLS_PER_SAMPLE} \
optimized_temporary_vec_allocations_per_sample=0 result_allocations_per_sample={CALLS_PER_SAMPLE} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        PAYLOADS.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "direct payload join P95 {optimized_p95_ns}ns must be at most 70% of temporary-Vec join P95 {legacy_p95_ns}ns"
    );
}

fn legacy_join_drag_payloads(accepts: &[UiDragPayloadKind]) -> String {
    accepts
        .iter()
        .map(|kind| kind.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

fn measure(join: fn(&[UiDragPayloadKind]) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..CALLS_PER_SAMPLE {
        checksum ^= black_box(join(black_box(&PAYLOADS))).len();
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

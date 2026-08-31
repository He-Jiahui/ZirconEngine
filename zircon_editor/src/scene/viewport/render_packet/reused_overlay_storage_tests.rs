use std::hint::black_box;
use std::time::Instant;

use super::*;

const ELEMENT_COUNT: usize = 16 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 128;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hi_editor201_preserves_overlay_replacement_semantics() {
    let mut target = Vec::with_capacity(8);
    target.push("stale".to_string());
    let retained_capacity = target.capacity();
    let source = [
        "move".to_string(),
        "rotate".to_string(),
        "scale".to_string(),
    ];

    replace_cloned_values(&mut target, &source);

    assert_eq!(target.as_slice(), source.as_slice());
    assert_eq!(target.capacity(), retained_capacity);
}

#[test]
fn optimization_batch_20260826hi_editor201_reuses_overlay_vector_storage() {
    let source = include_str!("../render_packet.rs");
    let start = source
        .find("pub(in crate::scene::viewport) fn apply_interaction_overlays(")
        .expect("apply_interaction_overlays function");
    let end = source[start..]
        .find("\npub(in crate::scene::viewport) fn build_scene_gizmos")
        .map(|offset| start + offset)
        .expect("build_scene_gizmos boundary");
    let body = &source[start..end];

    assert_eq!(body.matches("replace_cloned_values(").count(), 3);
    assert!(body.contains("target.clear()"));
    assert!(body.contains("target.extend_from_slice(source)"));
    assert!(!body.contains(".iter().cloned().collect()"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hi_editor201_reused_overlay_storage_release_benchmark() {
    let source = (0..ELEMENT_COUNT)
        .map(|value| value as u64)
        .collect::<Vec<_>>();
    let mut legacy = Vec::new();
    let mut optimized = Vec::new();

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                legacy_replace_cloned_values(black_box(&mut legacy), black_box(&source));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                replace_cloned_values(black_box(&mut optimized), black_box(&source));
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }
    assert_eq!(legacy, optimized);

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "EDITOR201_REUSED_OVERLAY_VECTOR_STORAGE_BENCH_V1 \
         element_count={ELEMENT_COUNT} operations_per_sample={OPERATIONS_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
         optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_replace_cloned_values<T: Clone>(target: &mut Vec<T>, source: &[T]) {
    *target = source.to_vec();
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

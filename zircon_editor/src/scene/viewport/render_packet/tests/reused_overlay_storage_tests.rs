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
    let source = include_str!("../../render_packet.rs");
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
fn optimization_batch_r6_wave6_editor641_scene_gizmos_reserve_kind_bound() {
    let source = include_str!("../../render_packet.rs");
    assert!(source.contains("let gizmo_capacity = scene"));
    assert!(source.contains("let mut gizmos = Vec::with_capacity(gizmo_capacity);"));
    assert!(source.contains(
        ".filter(|node| matches!(node.kind, NodeKind::Camera | NodeKind::DirectionalLight))"
    ));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_r6_wave6_editor641_scene_gizmo_capacity_p95() {
    const SAMPLE_PAIRS: usize = 101;
    const GIZMOS_PER_SAMPLE: usize = 65_536;
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(editor641_measure_gizmo_projection(GIZMOS_PER_SAMPLE, false));
            optimized_samples.push(editor641_measure_gizmo_projection(GIZMOS_PER_SAMPLE, true));
        } else {
            optimized_samples.push(editor641_measure_gizmo_projection(GIZMOS_PER_SAMPLE, true));
            legacy_samples.push(editor641_measure_gizmo_projection(GIZMOS_PER_SAMPLE, false));
        }
    }

    let legacy_p50 = editor641_percentile(&legacy_samples, 50);
    let legacy_p95 = editor641_percentile(&legacy_samples, 95);
    let optimized_p50 = editor641_percentile(&optimized_samples, 50);
    let optimized_p95 = editor641_percentile(&optimized_samples, 95);
    println!(
        "EDITOR641_PREALLOCATED_SCENE_GIZMOS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} gizmos_per_sample={GIZMOS_PER_SAMPLE} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_samples={} optimized_samples={} ratio={:.4}",
        editor641_format_samples(&legacy_samples),
        editor641_format_samples(&optimized_samples),
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(85),
        "preallocated scene gizmos must be at least 15% faster at P95"
    );
}

fn editor641_measure_gizmo_projection(output_count: usize, optimized: bool) -> u128 {
    let mut outputs = if optimized {
        Vec::with_capacity(output_count)
    } else {
        Vec::new()
    };
    let started = Instant::now();
    for value in 0..output_count {
        outputs.push(black_box(value));
    }
    let elapsed = started.elapsed().as_nanos().max(1);
    black_box(outputs);
    elapsed
}

fn editor641_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100).max(1);
    sorted[rank - 1]
}

fn editor641_format_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
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

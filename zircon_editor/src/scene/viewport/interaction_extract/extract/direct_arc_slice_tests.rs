use std::hint::black_box;
use std::time::Instant;

use super::*;

const MESH_COUNT: usize = 16 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 64;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hl_editor204_preserves_arc_slice_clone_semantics() {
    let source = vec!["mesh-a".to_string(), "mesh-b".to_string()];
    let cloned = cloned_arc_slice(&source);

    assert_eq!(cloned.as_ref(), source.as_slice());
    assert_eq!(Arc::strong_count(&cloned), 1);
    assert_eq!(source.len(), 2);
}

#[test]
fn optimization_batch_20260826hl_editor204_clones_render_meshes_directly_into_arc() {
    let source = include_str!("../extract.rs");
    assert!(source.contains("render_meshes: cloned_arc_slice(render_meshes)"));
    assert!(source.contains("Arc::from(source)"));
    assert!(!source.contains("render_meshes.to_vec().into()"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hl_editor204_direct_arc_slice_clone_release_benchmark() {
    let source = (0..MESH_COUNT)
        .map(|value| value as u64)
        .collect::<Vec<_>>();

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_cloned_arc_slice(black_box(&source)));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(cloned_arc_slice(black_box(&source)));
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

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "EDITOR204_DIRECT_ARC_SLICE_CLONE_BENCH_V1 \
         mesh_count={MESH_COUNT} operations_per_sample={OPERATIONS_PER_SAMPLE} \
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

fn legacy_cloned_arc_slice<T: Clone>(source: &[T]) -> Arc<[T]> {
    Arc::from(source.to_vec())
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

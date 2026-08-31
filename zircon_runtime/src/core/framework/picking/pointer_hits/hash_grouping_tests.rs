use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::core::framework::picking::{HitData, HitTarget};

const SAMPLE_PAIRS: usize = 21;
const POINTER_COUNT: usize = 4_096;
const HITS_PER_POINTER: usize = 16;

fn hit(owner: u64, depth: Real) -> HitRecord {
    HitRecord::new(
        HitTarget::renderable(owner),
        HitData::new(0, depth, None, None),
    )
}

fn benchmark_outputs() -> Vec<PointerHits> {
    (0..POINTER_COUNT * HITS_PER_POINTER)
        .map(|index| {
            let pointer_index = (index * 257) % POINTER_COUNT;
            PointerHits::new(
                PointerId::new(pointer_index as u64),
                vec![hit(index as u64, (index % HITS_PER_POINTER) as Real)],
                (index % 7) as Real,
            )
        })
        .collect()
}

fn legacy_sorted_hits_by_pointer(outputs: &[PointerHits]) -> BTreeMap<PointerId, Vec<HitRecord>> {
    let mut indexed_by_pointer = BTreeMap::<PointerId, Vec<IndexedHit>>::new();
    for (output_index, output) in outputs.iter().enumerate() {
        indexed_by_pointer
            .entry(output.pointer)
            .or_default()
            .extend(
                output
                    .hits
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(|(hit_index, hit)| (output_index, hit_index, output.order, hit)),
            );
    }
    indexed_by_pointer
        .into_iter()
        .map(|(pointer, mut indexed)| {
            sort_indexed_hits(&mut indexed);
            let hits = indexed.into_iter().map(|(_, _, _, hit)| hit).collect();
            (pointer, hits)
        })
        .collect()
}

#[test]
fn runtime47_batch_pointer_hash_groups_preserve_sorted_results() {
    let outputs = vec![
        PointerHits::new(PointerId::new(9), vec![hit(1, 4.0)], 1.0),
        PointerHits::new(PointerId::new(2), vec![hit(2, 8.0)], 2.0),
        PointerHits::new(PointerId::new(9), vec![hit(3, 2.0)], 3.0),
        PointerHits::new(PointerId::new(2), vec![hit(4, 1.0)], 1.0),
    ];

    assert_eq!(
        sorted_hits_by_pointer(&outputs),
        legacy_sorted_hits_by_pointer(&outputs)
    );
    assert_eq!(
        sorted_hits_by_pointer(&outputs)
            .keys()
            .map(|pointer| pointer.raw())
            .collect::<Vec<_>>(),
        vec![2, 9]
    );
}

#[test]
fn runtime47_batch_pointer_grouping_keeps_hash_private_and_output_ordered() {
    let source = include_str!("../pointer_hits.rs");
    let implementation = source
        .split("pub(super) fn sorted_hits_by_pointer")
        .nth(1)
        .and_then(|body| body.split("type IndexedHit").next())
        .expect("grouping implementation");

    assert!(implementation.contains("HashMap::<PointerId, Vec<IndexedHit>>::with_capacity"));
    assert!(implementation.contains("collect::<BTreeMap<_, _>>()"));
    assert!(!implementation.contains("BTreeMap::<PointerId, Vec<IndexedHit>>::new"));
}

fn measure(
    outputs: &[PointerHits],
    project: impl Fn(&[PointerHits]) -> BTreeMap<PointerId, Vec<HitRecord>>,
) -> u128 {
    let started = Instant::now();
    let projected = project(black_box(outputs));
    black_box(projected.len());
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
#[ignore = "release-only pointer hit grouping benchmark"]
fn runtime47_batch_pointer_hash_grouping_release_benchmark() {
    let outputs = benchmark_outputs();
    for _ in 0..4 {
        black_box(measure(&outputs, legacy_sorted_hits_by_pointer));
        black_box(measure(&outputs, sorted_hits_by_pointer));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&outputs, legacy_sorted_hits_by_pointer));
            optimized_samples.push(measure(&outputs, sorted_hits_by_pointer));
        } else {
            optimized_samples.push(measure(&outputs, sorted_hits_by_pointer));
            legacy_samples.push(measure(&outputs, legacy_sorted_hits_by_pointer));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME47_POINTER_HIT_HASH_GROUPING_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
pointer_count={POINTER_COUNT} hits_per_pointer={HITS_PER_POINTER} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "hash grouping must reduce P95 by at least 30%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

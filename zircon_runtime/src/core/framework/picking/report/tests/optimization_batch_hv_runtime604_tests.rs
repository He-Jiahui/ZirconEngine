use std::hint::black_box;
use std::time::Instant;

use super::*;

fn pointer_counts(pointers: &[PointerId]) -> BTreeMap<PointerId, usize> {
    let mut counts = BTreeMap::new();
    for pointer in pointers {
        *counts.entry(*pointer).or_insert(0) += 1;
    }
    counts
}

fn output_counts(pointers: &[PointerId]) -> BTreeMap<PointerId, (usize, usize)> {
    let mut counts = BTreeMap::new();
    for pointer in pointers {
        let count = counts.entry(*pointer).or_insert((0, 0));
        count.0 += 1;
    }
    counts
}

fn legacy_pointer_ids(
    ray_pointers: &[PointerId],
    output_pointers: &[PointerId],
) -> BTreeSet<PointerId> {
    ray_pointers
        .iter()
        .chain(output_pointers)
        .copied()
        .collect()
}

#[test]
fn optimization_batch_hv_runtime604_index_union_preserves_sorted_pointer_ids() {
    let ray_counts = pointer_counts(&[PointerId::new(7), PointerId::new(2), PointerId::new(7)]);
    let output_counts = output_counts(&[PointerId::new(9), PointerId::new(2), PointerId::new(9)]);

    assert_eq!(
        report_pointer_ids(&ray_counts, &output_counts)
            .into_iter()
            .collect::<Vec<_>>(),
        [PointerId::new(2), PointerId::new(7), PointerId::new(9)]
    );
}

#[test]
fn optimization_batch_hv_runtime604_pointer_ids_reuse_existing_count_indexes() {
    let source = include_str!("../../report.rs");
    let assembly = source
        .split("pub(super) fn from_ray_map_outputs_and_sorted_hits")
        .nth(1)
        .expect("report assembly")
        .split("pub fn pointer(&self")
        .next()
        .expect("bounded report assembly");
    let helper = source
        .split("fn report_pointer_ids")
        .nth(1)
        .expect("pointer id helper")
        .split("fn ray_count_by_pointer")
        .next()
        .expect("bounded pointer id helper");

    assert!(
        assembly.contains("report_pointer_ids(&ray_count_by_pointer, &output_counts_by_pointer)")
    );
    assert!(helper.contains("ray_count_by_pointer"));
    assert!(helper.contains("output_counts_by_pointer"));
    assert_eq!(helper.matches(".keys()").count(), 2);
    assert!(!helper.contains("ray_map.iter()"));
    assert!(!helper.contains("outputs.iter()"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_hv_runtime604_pointer_id_index_union_performance_evidence() {
    const UNIQUE_POINTERS: usize = 64;
    const RAY_VISITS: usize = 65_536;
    const OUTPUT_VISITS: usize = 65_536;
    const SAMPLE_PAIRS: usize = 17;
    let ray_pointers = (0..RAY_VISITS)
        .map(|index| PointerId::new((index % UNIQUE_POINTERS) as u64))
        .collect::<Vec<_>>();
    let output_pointers = (0..OUTPUT_VISITS)
        .map(|index| PointerId::new(((index * 17) % UNIQUE_POINTERS) as u64))
        .collect::<Vec<_>>();
    let ray_counts = pointer_counts(&ray_pointers);
    let output_counts = output_counts(&output_pointers);
    let measure_legacy = || {
        let started = Instant::now();
        black_box(legacy_pointer_ids(
            black_box(&ray_pointers),
            black_box(&output_pointers),
        ));
        started.elapsed().as_nanos().max(1)
    };
    let measure_indexed = || {
        let started = Instant::now();
        black_box(report_pointer_ids(
            black_box(&ray_counts),
            black_box(&output_counts),
        ));
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_legacy());
        black_box(measure_indexed());
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            indexed_samples.push(measure_indexed());
        } else {
            indexed_samples.push(measure_indexed());
            legacy_samples.push(measure_legacy());
        }
    }
    legacy_samples.sort_unstable();
    indexed_samples.sort_unstable();
    let legacy_p50 = legacy_samples[8];
    let legacy_p95 = legacy_samples[16];
    let indexed_p50 = indexed_samples[8];
    let indexed_p95 = indexed_samples[16];
    println!(
        "RUNTIME604_POINTER_ID_INDEX_UNION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 indexed_first_pairs=8 unique_pointers={UNIQUE_POINTERS} ray_visits={RAY_VISITS} output_visits={OUTPUT_VISITS} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} indexed_p50_ns={indexed_p50} indexed_p95_ns={indexed_p95} pointer_id_visits=131072->128 target_ratio_bp=1000"
    );
    assert!(
        indexed_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(1_000),
        "indexed pointer union P95 {indexed_p95} ns exceeded 10% of legacy {legacy_p95} ns"
    );
}

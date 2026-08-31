use std::{hint::black_box, time::Instant};

use super::*;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime08c_batch_animation_reference_dedup_preserves_first_seen_owned_order() {
    let first = asset_reference("res://animation/first.graph.zranim");
    let second = asset_reference("res://animation/second.graph.zranim");
    let duplicate_first = first.clone();
    let mut collector = DirectReferenceCollector::with_capacity(3);

    collector.push(&first);
    collector.push(&second);
    collector.push(&duplicate_first);
    let references = collector.into_references();

    drop((first, second, duplicate_first));
    assert_eq!(references.len(), 2);
    assert_eq!(
        references[0].locator.to_string(),
        "res://animation/first.graph.zranim"
    );
    assert_eq!(
        references[1].locator.to_string(),
        "res://animation/second.graph.zranim"
    );
}

#[test]
fn runtime08c_batch_animation_reference_uses_borrowed_hash_dedup() {
    let reference_source = include_str!("../reference.rs");
    let graph_source = include_str!("../graph.rs");
    let state_kind_source = include_str!("../state_kind.rs");
    let state_machine_source = include_str!("../state_machine.rs");

    assert!(reference_source.contains("HashSet<&'a AssetReference>"));
    assert!(reference_source.contains("if self.seen.insert(reference)"));
    assert!(reference_source.contains("self.references.push(reference.clone())"));
    assert!(!reference_source.contains(".iter()\n        .any("));
    assert!(graph_source.contains("DirectReferenceCollector::with_capacity"));
    assert!(state_kind_source.contains("DirectReferenceCollector<'a>"));
    assert!(state_machine_source.contains("DirectReferenceCollector::with_capacity"));
    assert!(!graph_source.contains("push_unique_reference"));
    assert!(!state_kind_source.contains("push_unique_reference"));
    assert!(!state_machine_source.contains("push_unique_reference"));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime08c_batch_animation_reference_borrowed_dedup_p95() {
    const REFERENCE_COUNT: usize = 2_048;
    const BUILDS: usize = 2;
    let references = (0..REFERENCE_COUNT)
        .map(|index| asset_reference(&format!("res://animation/reference-{index:04}.zranim")))
        .collect::<Vec<_>>();
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);

    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(BUILDS, || {
                legacy_collect(black_box(&references))
            }));
            optimized_ns.push(measure_ns(BUILDS, || {
                borrowed_collect(black_box(&references))
            }));
        } else {
            optimized_ns.push(measure_ns(BUILDS, || {
                borrowed_collect(black_box(&references))
            }));
            legacy_ns.push(measure_ns(BUILDS, || {
                legacy_collect(black_box(&references))
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns,
        "borrowed animation reference dedup P95 must be at least 90% below repeated vector scans: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME08C_ANIMATION_REFERENCE_BORROWED_DEDUP_BENCH_V1 references={REFERENCE_COUNT} builds_per_sample={BUILDS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_reference_comparisons_per_sample={} optimized_hash_inserts_per_sample={} legacy_output_clones_per_sample={} optimized_output_clones_per_sample={} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        REFERENCE_COUNT * (REFERENCE_COUNT - 1) / 2 * BUILDS,
        REFERENCE_COUNT * BUILDS,
        REFERENCE_COUNT * BUILDS,
        REFERENCE_COUNT * BUILDS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn borrowed_collect(references: &[AssetReference]) -> usize {
    let mut collector = DirectReferenceCollector::with_capacity(references.len());
    for reference in references {
        collector.push(reference);
    }
    black_box(collector.into_references()).len()
}

fn legacy_collect(references: &[AssetReference]) -> usize {
    let mut unique = Vec::<AssetReference>::with_capacity(references.len());
    for reference in references {
        if !unique.iter().any(|existing| existing == reference) {
            unique.push(reference.clone());
        }
    }
    black_box(unique).len()
}

fn asset_reference(locator: &str) -> AssetReference {
    AssetReference::from_locator(ResourceLocator::parse(locator).expect("asset locator"))
}

fn measure_ns(iterations: usize, mut operation: impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box(operation()));
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

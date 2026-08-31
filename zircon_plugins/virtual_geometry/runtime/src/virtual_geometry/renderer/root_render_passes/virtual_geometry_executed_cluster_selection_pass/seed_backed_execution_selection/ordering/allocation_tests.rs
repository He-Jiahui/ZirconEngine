use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::render::RenderVirtualGeometryCluster;

use super::{finalize_seed_backed_cluster_ordering, SeedBackedClusterOrdering};

const BENCH_CLUSTER_COUNT: usize = 4_096;
const BENCH_ENTITY_COUNT: usize = 8;
const CHECKS_PER_SAMPLE: usize = 8;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn seed_backed_ordering_preserves_unique_ordinals_after_duplicate_clusters() {
    let clusters_by_entity = HashMap::from([
        (7, vec![cluster(7, 30), cluster(7, 10), cluster(7, 10)]),
        (8, vec![cluster(8, 4), cluster(8, 2)]),
    ]);

    let optimized = finalize_seed_backed_cluster_ordering(clusters_by_entity.clone());
    let legacy = legacy_finalize(clusters_by_entity);

    assert_eq!(ordering_entries(&optimized), ordering_entries(&legacy));
    assert_eq!(optimized.len(), 4);
    assert_eq!(optimized[&(7, 10)].cluster_ordinal(), 0);
    assert_eq!(optimized[&(7, 30)].cluster_ordinal(), 1);
    assert_eq!(optimized[&(7, 10)].entity_cluster_total_count(), 2);
}

#[test]
#[ignore = "release-only seed-backed ordering benchmark"]
fn seed_backed_ordering_release_benchmark_evidence() {
    let clusters_per_entity = BENCH_CLUSTER_COUNT / BENCH_ENTITY_COUNT;
    let clusters_by_entity = (0..BENCH_ENTITY_COUNT)
        .map(|entity_index| {
            let entity = entity_index as u64;
            let clusters = (0..clusters_per_entity)
                .map(|cluster_index| {
                    let shuffled = (cluster_index * 197) % clusters_per_entity;
                    cluster(entity, shuffled as u32)
                })
                .collect::<Vec<_>>();
            (entity, clusters)
        })
        .collect::<HashMap<_, _>>();
    assert_eq!(
        ordering_entries(&legacy_finalize(clusters_by_entity.clone())),
        ordering_entries(&finalize_seed_backed_cluster_ordering(
            clusters_by_entity.clone()
        ))
    );

    for _ in 0..4 {
        black_box(measure_legacy(&clusters_by_entity));
        black_box(measure_optimized(&clusters_by_entity));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&clusters_by_entity));
            optimized_samples.push(measure_optimized(&clusters_by_entity));
        } else {
            optimized_samples.push(measure_optimized(&clusters_by_entity));
            legacy_samples.push(measure_legacy(&clusters_by_entity));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=seed_backed_ordering_sort \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
entity_count={BENCH_ENTITY_COUNT} cluster_count={BENCH_CLUSTER_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_sort=stable_with_scratch optimized_sort=unstable_in_place \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(20) <= legacy_p95_ns.saturating_mul(19),
        "in-place seed-backed ordering must reduce P95 by at least 5%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(clusters_by_entity: &HashMap<u64, Vec<RenderVirtualGeometryCluster>>) -> u128 {
    let inputs = (0..CHECKS_PER_SAMPLE)
        .map(|_| clusters_by_entity.clone())
        .collect::<Vec<_>>();
    let started = Instant::now();
    for input in inputs {
        black_box(legacy_finalize(input));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(clusters_by_entity: &HashMap<u64, Vec<RenderVirtualGeometryCluster>>) -> u128 {
    let inputs = (0..CHECKS_PER_SAMPLE)
        .map(|_| clusters_by_entity.clone())
        .collect::<Vec<_>>();
    let started = Instant::now();
    for input in inputs {
        black_box(finalize_seed_backed_cluster_ordering(input));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_finalize(
    clusters_by_entity: HashMap<u64, Vec<RenderVirtualGeometryCluster>>,
) -> HashMap<(u64, u32), SeedBackedClusterOrdering> {
    let mut ordering = HashMap::new();
    for (entity, mut clusters) in clusters_by_entity {
        clusters.sort_by_key(|cluster| cluster.cluster_id);
        clusters.dedup_by_key(|cluster| cluster.cluster_id);
        let entity_cluster_total_count = clusters.len().max(1);
        for (cluster_ordinal, cluster) in clusters.into_iter().enumerate() {
            ordering.insert(
                (entity, cluster.cluster_id),
                SeedBackedClusterOrdering::new(
                    u32::try_from(cluster_ordinal).unwrap_or(u32::MAX),
                    entity_cluster_total_count,
                ),
            );
        }
    }
    ordering
}

fn ordering_entries(
    ordering: &HashMap<(u64, u32), SeedBackedClusterOrdering>,
) -> BTreeMap<(u64, u32), (u32, usize)> {
    ordering
        .iter()
        .map(|(key, value)| {
            (
                *key,
                (value.cluster_ordinal(), value.entity_cluster_total_count()),
            )
        })
        .collect()
}

fn cluster(entity: u64, cluster_id: u32) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        entity,
        cluster_id,
        ..RenderVirtualGeometryCluster::default()
    }
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

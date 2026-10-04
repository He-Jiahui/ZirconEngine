use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract, RenderVirtualGeometryInstance,
};

use super::OverlayClusterLookup;

const BENCHMARK_CLUSTER_COUNT: usize = 512;
const BENCHMARK_LOOKUP_COUNT: usize = 2_048;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn overlay_lookup_preserves_stable_entity_order_and_first_instance_owner() {
    let extract = RenderVirtualGeometryExtract {
        clusters: vec![
            cluster(999, 3, 10),
            cluster(999, 1, 11),
            cluster(999, 3, 12),
        ],
        instances: vec![instance(7, 0, 2), instance(7, 2, 1), instance(8, 2, 1)],
        ..Default::default()
    };

    let lookup = OverlayClusterLookup::new(&extract);

    assert_eq!(
        lookup
            .clusters_for_entity(7)
            .iter()
            .map(|cluster| (cluster.cluster_id, cluster.page_id))
            .collect::<Vec<_>>(),
        vec![(1, 11), (3, 10)]
    );
    assert_eq!(lookup.cluster_ordinal(7, 1), Some(0));
    assert_eq!(lookup.cluster_ordinal(7, 3), Some(1));
    assert_eq!(lookup.instance_index(7, 3), Some(0));
    assert_eq!(lookup.instance_index(8, 3), Some(2));
}

#[test]
fn cached_overlay_cluster_lookup_performance_contract() {
    let extract = RenderVirtualGeometryExtract {
        clusters: (0..BENCHMARK_CLUSTER_COUNT as u32)
            .rev()
            .map(|cluster_id| cluster(7, cluster_id, cluster_id))
            .collect(),
        instances: vec![instance(7, 0, BENCHMARK_CLUSTER_COUNT as u32)],
        ..Default::default()
    };
    let legacy = || {
        let mut checksum = 0_u64;
        for lookup_index in 0..BENCHMARK_LOOKUP_COUNT {
            let cluster_id = (lookup_index % BENCHMARK_CLUSTER_COUNT) as u32;
            let clusters = legacy_clusters_for_entity(black_box(&extract), 7);
            checksum ^= clusters
                .iter()
                .position(|cluster| cluster.cluster_id == cluster_id)
                .unwrap_or_default() as u64;
            checksum ^= u64::from(
                legacy_instance_index_for_cluster(&extract, 7, cluster_id).unwrap_or_default(),
            );
        }
        black_box(checksum);
    };
    let optimized = || {
        let lookup = OverlayClusterLookup::new(black_box(&extract));
        let mut checksum = 0_u64;
        for lookup_index in 0..BENCHMARK_LOOKUP_COUNT {
            let cluster_id = (lookup_index % BENCHMARK_CLUSTER_COUNT) as u32;
            checksum ^= u64::from(lookup.cluster_ordinal(7, cluster_id).unwrap_or_default());
            checksum ^= u64::from(lookup.instance_index(7, cluster_id).unwrap_or_default());
        }
        black_box(checksum);
    };

    legacy();
    optimized();
    let (legacy_samples, optimized_samples) = paired_samples(legacy, optimized);
    let legacy_p50 = nearest_rank(&legacy_samples, 50).as_nanos();
    let legacy_p95 = nearest_rank(&legacy_samples, 95).as_nanos();
    let optimized_p50 = nearest_rank(&optimized_samples, 50).as_nanos();
    let optimized_p95 = nearest_rank(&optimized_samples, 95).as_nanos();

    println!(
        "PERF_RESULT plugins17_cached_overlay_cluster_lookup clusters={BENCHMARK_CLUSTER_COUNT} lookups={BENCHMARK_LOOKUP_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_entity_vec_allocations_per_sample={BENCHMARK_LOOKUP_COUNT} optimized_entity_vec_allocations_per_sample=1 legacy_entity_sorts_per_sample={BENCHMARK_LOOKUP_COUNT} optimized_entity_sorts_per_sample=1 legacy_instance_scans_per_sample={BENCHMARK_LOOKUP_COUNT} optimized_instance_scans_per_sample=1 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_p50} optimized_ns={optimized_p50}"
    );
    assert!(
        optimized_p95 < legacy_p95,
        "cached overlay lookup must beat repeated collection and scans: legacy_p95={legacy_p95}ns optimized_p95={optimized_p95}ns"
    );
}

fn cluster(entity: u64, cluster_id: u32, page_id: u32) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        entity,
        cluster_id,
        page_id,
        ..Default::default()
    }
}

fn instance(entity: u64, cluster_offset: u32, cluster_count: u32) -> RenderVirtualGeometryInstance {
    RenderVirtualGeometryInstance {
        entity,
        cluster_offset,
        cluster_count,
        ..Default::default()
    }
}

fn legacy_clusters_for_entity(
    extract: &RenderVirtualGeometryExtract,
    entity: u64,
) -> Vec<RenderVirtualGeometryCluster> {
    let mut clusters = extract
        .instances
        .iter()
        .filter(|instance| instance.entity == entity)
        .flat_map(|instance| {
            let start = instance.cluster_offset as usize;
            let end = start.saturating_add(instance.cluster_count as usize);
            extract
                .clusters
                .get(start..end)
                .into_iter()
                .flatten()
                .copied()
        })
        .collect::<Vec<_>>();
    clusters.sort_by_key(|cluster| cluster.cluster_id);
    clusters.dedup_by_key(|cluster| cluster.cluster_id);
    clusters
}

fn legacy_instance_index_for_cluster(
    extract: &RenderVirtualGeometryExtract,
    entity: u64,
    cluster_id: u32,
) -> Option<u32> {
    extract
        .instances
        .iter()
        .enumerate()
        .find(|(_, instance)| {
            if instance.entity != entity {
                return false;
            }
            let start = instance.cluster_offset as usize;
            let end = start.saturating_add(instance.cluster_count as usize);
            extract
                .clusters
                .get(start..end)
                .into_iter()
                .flatten()
                .any(|cluster| cluster.cluster_id == cluster_id)
        })
        .and_then(|(instance_index, _)| u32::try_from(instance_index).ok())
}

fn paired_samples(legacy: impl Fn(), optimized: impl Fn()) -> (Vec<Duration>, Vec<Duration>) {
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure(&legacy));
            optimized_samples.push(measure(&optimized));
        } else {
            optimized_samples.push(measure(&optimized));
            legacy_samples.push(measure(&legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure(run: impl Fn()) -> Duration {
    let started = Instant::now();
    run();
    started.elapsed()
}

fn nearest_rank(samples: &[Duration], percentile: usize) -> Duration {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = (ordered.len() * percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
};

use super::build_node_and_cluster_cull_cluster_work_items_from_clusters;
use crate::virtual_geometry::types::VirtualGeometryNodeAndClusterCullClusterWorkItem;

const BENCH_CLUSTER_COUNT: usize = 4_096;
const BENCH_INSTANCE_COUNT: usize = 256;
const CHECKS_PER_SAMPLE: usize = 64;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn bounded_cluster_worklist_preserves_instance_and_cluster_order() {
    let clusters = (0..8)
        .map(|cluster_index| RenderVirtualGeometryCluster {
            hierarchy_node_id: Some(100 + cluster_index),
            ..RenderVirtualGeometryCluster::default()
        })
        .collect::<Vec<_>>();
    let instance_work_items = [instance_work_item(0, 2), instance_work_item(4, 3)];

    let optimized = build_node_and_cluster_cull_cluster_work_items_from_clusters(
        &clusters,
        &instance_work_items,
        3,
    );
    let legacy = legacy_build_cluster_work_items(&clusters, &instance_work_items, 3);

    assert_eq!(optimized, legacy);
    assert_eq!(
        optimized
            .iter()
            .map(|item| item.cluster_array_index)
            .collect::<Vec<_>>(),
        vec![0, 1, 4]
    );
    assert!(optimized.capacity() >= optimized.len());
}

#[test]
#[ignore = "release-only bounded cluster worklist benchmark"]
fn bounded_cluster_worklist_release_benchmark_evidence() {
    let clusters = (0..BENCH_CLUSTER_COUNT)
        .map(|cluster_index| RenderVirtualGeometryCluster {
            hierarchy_node_id: Some(cluster_index as u32),
            ..RenderVirtualGeometryCluster::default()
        })
        .collect::<Vec<_>>();
    let clusters_per_instance = BENCH_CLUSTER_COUNT / BENCH_INSTANCE_COUNT;
    let instance_work_items = (0..BENCH_INSTANCE_COUNT)
        .map(|instance_index| {
            instance_work_item(
                (instance_index * clusters_per_instance) as u32,
                clusters_per_instance as u32,
            )
        })
        .collect::<Vec<_>>();
    let limit = BENCH_CLUSTER_COUNT as u32;
    assert_eq!(
        legacy_build_cluster_work_items(&clusters, &instance_work_items, limit),
        build_node_and_cluster_cull_cluster_work_items_from_clusters(
            &clusters,
            &instance_work_items,
            limit,
        )
    );

    for _ in 0..4 {
        black_box(measure_legacy(&clusters, &instance_work_items, limit));
        black_box(measure_optimized(&clusters, &instance_work_items, limit));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&clusters, &instance_work_items, limit));
            optimized_samples.push(measure_optimized(&clusters, &instance_work_items, limit));
        } else {
            optimized_samples.push(measure_optimized(&clusters, &instance_work_items, limit));
            legacy_samples.push(measure_legacy(&clusters, &instance_work_items, limit));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=bounded_cluster_worklist_capacity \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
instance_count={BENCH_INSTANCE_COUNT} cluster_count={BENCH_CLUSTER_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_preallocated_items=0 optimized_preallocated_items={BENCH_CLUSTER_COUNT} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(9),
        "bounded cluster worklist must reduce P95 by at least 10%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(
    clusters: &[RenderVirtualGeometryCluster],
    instance_work_items: &[RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem],
    limit: u32,
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_build_cluster_work_items(
            black_box(clusters),
            black_box(instance_work_items),
            limit,
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(
    clusters: &[RenderVirtualGeometryCluster],
    instance_work_items: &[RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem],
    limit: u32,
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(
            build_node_and_cluster_cull_cluster_work_items_from_clusters(
                black_box(clusters),
                black_box(instance_work_items),
                limit,
            ),
        );
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_build_cluster_work_items(
    clusters: &[RenderVirtualGeometryCluster],
    instance_work_items: &[RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem],
    limit: u32,
) -> Vec<VirtualGeometryNodeAndClusterCullClusterWorkItem> {
    instance_work_items
        .iter()
        .flat_map(|work_item| {
            (0..work_item.cluster_count).map(move |cluster_local_index| {
                let cluster_array_index =
                    work_item.cluster_offset.saturating_add(cluster_local_index);
                VirtualGeometryNodeAndClusterCullClusterWorkItem {
                    instance_index: work_item.instance_index,
                    entity: work_item.entity,
                    cluster_array_index,
                    hierarchy_node_id: clusters
                        .get(cluster_array_index as usize)
                        .and_then(|cluster| cluster.hierarchy_node_id),
                    cluster_budget: work_item.cluster_budget,
                    page_budget: work_item.page_budget,
                    forced_mip: work_item.forced_mip,
                }
            })
        })
        .take(limit as usize)
        .collect()
}

fn instance_work_item(
    cluster_offset: u32,
    cluster_count: u32,
) -> RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem {
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem {
        instance_index: cluster_offset,
        entity: 42 + u64::from(cluster_offset),
        cluster_offset,
        cluster_count,
        page_offset: 0,
        page_count: 0,
        cluster_budget: u32::MAX,
        page_budget: u32::MAX,
        forced_mip: None,
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

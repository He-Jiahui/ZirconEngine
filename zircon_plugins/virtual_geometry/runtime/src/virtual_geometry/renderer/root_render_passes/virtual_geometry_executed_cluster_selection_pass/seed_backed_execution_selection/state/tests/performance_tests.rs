use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

use crate::virtual_geometry::types::VirtualGeometryPrepareClusterState;
use zircon_runtime::core::framework::render::RenderVirtualGeometryCluster;

use super::{resolve_seed_backed_execution_cluster_state_and_lineage, seed_backed_cluster_state};

const BENCH_LINEAGE_DEPTH: usize = 64;
const CHECKS_PER_SAMPLE: usize = 4_096;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn combined_seed_state_walk_preserves_resolution_lineage_and_cycle_semantics() {
    let clusters_by_id = HashMap::from([
        (1, cluster(1, 10, None, 7)),
        (2, cluster(2, 20, Some(1), 7)),
        (3, cluster(3, 30, Some(2), 7)),
        (4, cluster(4, 40, Some(3), 7)),
    ]);
    let page_residency = HashMap::from([(10, true), (20, true), (30, false), (40, false)]);

    assert_eq!(
        resolve_seed_backed_execution_cluster_state_and_lineage(
            clusters_by_id[&4],
            &clusters_by_id,
            &page_residency,
            None,
        ),
        (
            clusters_by_id[&2],
            3,
            VirtualGeometryPrepareClusterState::PendingUpload,
            VirtualGeometryPrepareClusterState::Resident,
        )
    );

    let self_cycle = cluster(9, 90, Some(9), 7);
    let cycle_index = HashMap::from([(9, self_cycle)]);
    assert_eq!(
        resolve_seed_backed_execution_cluster_state_and_lineage(
            self_cycle,
            &cycle_index,
            &HashMap::new(),
            None,
        ),
        (
            self_cycle,
            1,
            VirtualGeometryPrepareClusterState::Missing,
            VirtualGeometryPrepareClusterState::Missing,
        )
    );
}

#[test]
#[ignore = "release-only combined seed state and lineage benchmark"]
fn combined_seed_state_walk_release_benchmark_evidence() {
    let clusters_by_id = (0..=BENCH_LINEAGE_DEPTH as u32)
        .map(|cluster_id| {
            (
                cluster_id,
                cluster(cluster_id, cluster_id, cluster_id.checked_sub(1), 7),
            )
        })
        .collect::<HashMap<_, _>>();
    let page_residency = HashMap::from([(0, true)]);
    let cluster = clusters_by_id[&(BENCH_LINEAGE_DEPTH as u32)];
    assert_eq!(
        resolve_seed_backed_execution_cluster_state_and_lineage(
            cluster,
            &clusters_by_id,
            &page_residency,
            None,
        ),
        legacy_resolution_state_and_lineage(cluster, &clusters_by_id, &page_residency, None)
    );

    for _ in 0..4 {
        black_box(measure_legacy(cluster, &clusters_by_id, &page_residency));
        black_box(measure_optimized(cluster, &clusters_by_id, &page_residency));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(cluster, &clusters_by_id, &page_residency));
            optimized_samples.push(measure_optimized(cluster, &clusters_by_id, &page_residency));
        } else {
            optimized_samples.push(measure_optimized(cluster, &clusters_by_id, &page_residency));
            legacy_samples.push(measure_legacy(cluster, &clusters_by_id, &page_residency));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=seed_state_lineage_single_walk \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
lineage_depth={BENCH_LINEAGE_DEPTH} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_parent_walks=2 legacy_cycle_sets=2 optimized_parent_walks=1 optimized_cycle_sets=1 \
legacy_state_lookups=duplicated optimized_state_lookups=reused \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(20) <= legacy_p95_ns.saturating_mul(13),
        "single-walk seed state and lineage must reduce P95 by at least 35%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(
    cluster: RenderVirtualGeometryCluster,
    clusters_by_id: &HashMap<u32, RenderVirtualGeometryCluster>,
    page_residency: &HashMap<u32, bool>,
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_resolution_state_and_lineage(
            black_box(cluster),
            black_box(clusters_by_id),
            black_box(page_residency),
            None,
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(
    cluster: RenderVirtualGeometryCluster,
    clusters_by_id: &HashMap<u32, RenderVirtualGeometryCluster>,
    page_residency: &HashMap<u32, bool>,
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(resolve_seed_backed_execution_cluster_state_and_lineage(
            black_box(cluster),
            black_box(clusters_by_id),
            black_box(page_residency),
            None,
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_resolution_state_and_lineage(
    cluster: RenderVirtualGeometryCluster,
    clusters_by_id: &HashMap<u32, RenderVirtualGeometryCluster>,
    page_residency: &HashMap<u32, bool>,
    forced_mip: Option<u8>,
) -> (
    RenderVirtualGeometryCluster,
    u32,
    VirtualGeometryPrepareClusterState,
    VirtualGeometryPrepareClusterState,
) {
    let submission_state = seed_backed_cluster_state(cluster.page_id, page_residency);
    let resolved_cluster = legacy_resolve_cluster(
        cluster,
        clusters_by_id,
        page_residency,
        forced_mip,
        submission_state,
    );
    let selected_state = seed_backed_cluster_state(resolved_cluster.page_id, page_residency);
    (
        resolved_cluster,
        legacy_lineage_depth(cluster, clusters_by_id),
        submission_state,
        selected_state,
    )
}

fn legacy_resolve_cluster(
    cluster: RenderVirtualGeometryCluster,
    clusters_by_id: &HashMap<u32, RenderVirtualGeometryCluster>,
    page_residency: &HashMap<u32, bool>,
    forced_mip: Option<u8>,
    submission_state: VirtualGeometryPrepareClusterState,
) -> RenderVirtualGeometryCluster {
    if forced_mip.is_some() || submission_state == VirtualGeometryPrepareClusterState::Resident {
        return cluster;
    }
    let mut current_parent_cluster_id = cluster.parent_cluster_id;
    let mut visited_cluster_ids = HashSet::from([cluster.cluster_id]);
    while let Some(parent_cluster_id) = current_parent_cluster_id {
        if !visited_cluster_ids.insert(parent_cluster_id) {
            break;
        }
        let Some(parent_cluster) = clusters_by_id.get(&parent_cluster_id).copied() else {
            break;
        };
        if parent_cluster.entity != cluster.entity {
            break;
        }
        if seed_backed_cluster_state(parent_cluster.page_id, page_residency)
            == VirtualGeometryPrepareClusterState::Resident
        {
            return parent_cluster;
        }
        current_parent_cluster_id = parent_cluster.parent_cluster_id;
    }
    cluster
}

fn legacy_lineage_depth(
    cluster: RenderVirtualGeometryCluster,
    clusters_by_id: &HashMap<u32, RenderVirtualGeometryCluster>,
) -> u32 {
    let mut depth = 0_u32;
    let mut current_parent_cluster_id = cluster.parent_cluster_id;
    let mut visited_cluster_ids = HashSet::new();
    while let Some(parent_cluster_id) = current_parent_cluster_id {
        if !visited_cluster_ids.insert(parent_cluster_id) {
            break;
        }
        depth = depth.saturating_add(1);
        current_parent_cluster_id = clusters_by_id
            .get(&parent_cluster_id)
            .and_then(|parent| parent.parent_cluster_id);
    }
    depth
}

fn cluster(
    cluster_id: u32,
    page_id: u32,
    parent_cluster_id: Option<u32>,
    entity: u64,
) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        entity,
        cluster_id,
        page_id,
        parent_cluster_id,
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

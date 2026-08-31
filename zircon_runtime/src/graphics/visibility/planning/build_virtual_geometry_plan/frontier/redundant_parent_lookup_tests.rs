use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::render::RenderVirtualGeometryCluster;

use super::refine_visible_cluster_frontier;

const CLUSTER_COUNT: usize = 256;
const CHECKS_PER_SAMPLE: usize = 10_000;
const SAMPLE_PAIRS: usize = 31;

fn legacy_children_by_parent(
    visible_clusters: &[RenderVirtualGeometryCluster],
) -> BTreeMap<u32, Vec<RenderVirtualGeometryCluster>> {
    let visible_by_id = visible_clusters
        .iter()
        .map(|cluster| (cluster.cluster_id, *cluster))
        .collect::<BTreeMap<_, _>>();
    let mut children_by_parent = BTreeMap::<u32, Vec<RenderVirtualGeometryCluster>>::new();
    for cluster in visible_clusters.iter().copied() {
        if let Some(parent_cluster_id) = cluster.parent_cluster_id {
            if visible_by_id.contains_key(&parent_cluster_id) {
                children_by_parent
                    .entry(parent_cluster_id)
                    .or_default()
                    .push(cluster);
            }
        }
    }
    children_by_parent
}

fn optimized_children_by_parent(
    visible_clusters: &[RenderVirtualGeometryCluster],
) -> BTreeMap<u32, Vec<RenderVirtualGeometryCluster>> {
    let mut children_by_parent = BTreeMap::<u32, Vec<RenderVirtualGeometryCluster>>::new();
    for cluster in visible_clusters.iter().copied() {
        if let Some(parent_cluster_id) = cluster.parent_cluster_id {
            children_by_parent
                .entry(parent_cluster_id)
                .or_default()
                .push(cluster);
        }
    }
    children_by_parent
}

fn measure(visible_clusters: &[RenderVirtualGeometryCluster], optimized: bool) -> u128 {
    let started = Instant::now();
    let mut evidence = 0_usize;
    for _ in 0..CHECKS_PER_SAMPLE {
        let children = if optimized {
            optimized_children_by_parent(black_box(visible_clusters))
        } else {
            legacy_children_by_parent(black_box(visible_clusters))
        };
        evidence = evidence.wrapping_add(children.len());
    }
    black_box(evidence);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn cluster(cluster_id: u32, parent_cluster_id: Option<u32>) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        cluster_id,
        parent_cluster_id,
        ..RenderVirtualGeometryCluster::default()
    }
}

#[test]
fn optimization_batch_20260830bd_runtime356_parent_lookup_elision_preserves_frontier() {
    let visible_clusters = vec![cluster(1, None), cluster(2, Some(1)), cluster(3, Some(99))];
    let legacy = legacy_children_by_parent(&visible_clusters);
    let optimized = optimized_children_by_parent(&visible_clusters);
    assert_eq!(legacy, optimized);
    let result = refine_visible_cluster_frontier(&visible_clusters, 2, None, None, None);
    assert_eq!(result.len(), 2);
}

#[test]
fn optimization_batch_20260830bd_runtime356_production_drops_redundant_parent_lookup() {
    let source = include_str!("refine_visible_cluster_frontier.rs");
    assert_eq!(source.matches("visible_by_id.contains_key").count(), 0);
    assert!(source.contains(".entry(parent_cluster_id)"));
}

#[test]
#[ignore = "managed performance gate"]
fn optimization_batch_20260830bd_runtime356_parent_lookup_benchmark() {
    let visible_clusters = (0..CLUSTER_COUNT as u32)
        .map(|cluster_id| cluster(cluster_id, (cluster_id > 0).then_some(cluster_id - 1)))
        .collect::<Vec<_>>();
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(&visible_clusters, false));
            candidate.push(measure(&visible_clusters, true));
        } else {
            candidate.push(measure(&visible_clusters, true));
            baseline.push(measure(&visible_clusters, false));
        }
    }
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "RUNTIME356_PARENT_LOOKUP_ELISION_BENCH_V1 baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_samples_ns={} candidate_samples_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns * 100 <= baseline_p95_ns * 70);
}

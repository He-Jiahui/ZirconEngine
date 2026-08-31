use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::core::framework::render::{RenderVirtualGeometryCluster, RenderVirtualGeometryInstance};

const CLUSTERS_PER_INSTANCE: usize = 32;
const INSTANCE_COUNT: usize = 256;
const CHECKS_PER_SAMPLE: usize = 1024;
const SAMPLE_PAIRS: usize = 31;
const STABLE_KEY: u64 = 17;

fn legacy_cluster_ids_from_instances(
    extract: &RenderVirtualGeometryExtract,
    stable_instance_key: u64,
) -> Vec<u32> {
    extract
        .instances
        .iter()
        .filter(|instance| stable_instance_key_for_instance(instance) == stable_instance_key)
        .flat_map(|instance| {
            let start = instance.cluster_offset as usize;
            let end = start.saturating_add(instance.cluster_count as usize);
            extract
                .clusters
                .get(start..end)
                .into_iter()
                .flatten()
                .map(|cluster| cluster.cluster_id)
        })
        .collect::<Vec<_>>()
}

fn fixture_extract() -> RenderVirtualGeometryExtract {
    let clusters = (0..INSTANCE_COUNT * CLUSTERS_PER_INSTANCE)
        .map(|cluster_id| RenderVirtualGeometryCluster {
            cluster_id: cluster_id as u32,
            ..RenderVirtualGeometryCluster::default()
        })
        .collect::<Vec<_>>();
    let instances = (0..INSTANCE_COUNT)
        .map(|index| RenderVirtualGeometryInstance {
            stable_instance_key: STABLE_KEY,
            cluster_offset: (index * CLUSTERS_PER_INSTANCE) as u32,
            cluster_count: CLUSTERS_PER_INSTANCE as u32,
            ..RenderVirtualGeometryInstance::default()
        })
        .collect::<Vec<_>>();
    RenderVirtualGeometryExtract {
        clusters,
        instances,
        ..RenderVirtualGeometryExtract::default()
    }
}

fn measure(extract: &RenderVirtualGeometryExtract, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut evidence = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        let cluster_ids = if optimized {
            cluster_ids_from_instances(black_box(extract), STABLE_KEY)
        } else {
            legacy_cluster_ids_from_instances(black_box(extract), STABLE_KEY)
        };
        evidence += cluster_ids.len();
        black_box(cluster_ids);
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

#[test]
fn optimization_batch_20260829bx_runtime351_segmented_cluster_extend_preserves_results() {
    let mut extract = fixture_extract();
    assert_eq!(
        cluster_ids_from_instances(&extract, STABLE_KEY),
        legacy_cluster_ids_from_instances(&extract, STABLE_KEY)
    );
    extract.instances[3].cluster_offset = u32::MAX;
    assert_eq!(
        cluster_ids_from_instances(&extract, STABLE_KEY),
        legacy_cluster_ids_from_instances(&extract, STABLE_KEY)
    );
}

#[test]
fn optimization_batch_20260829bx_runtime351_cluster_collection_extends_exact_slices() {
    let source = include_str!("../cluster_ids_for_entity.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    let function = production
        .split_once("fn cluster_ids_from_instances")
        .expect("instance collector")
        .1
        .split_once("fn stable_instance_key_for_instance")
        .expect("stable-key boundary")
        .0;
    assert!(function.contains("cluster_ids.extend"));
    assert!(function.contains("extract.clusters.get(start..end)"));
    assert!(!function.contains(".flat_map"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bx_runtime351_segmented_cluster_extend_bench() {
    let extract = fixture_extract();
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(&extract, false));
            candidate.push(measure(&extract, true));
        } else {
            candidate.push(measure(&extract, true));
            baseline.push(measure(&extract, false));
        }
    }
    let baseline_p50_ns = percentile(&baseline, 50);
    let candidate_p50_ns = percentile(&candidate, 50);
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "RUNTIME351_SEGMENTED_CLUSTER_EXTEND_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} instance_count={INSTANCE_COUNT} clusters_per_instance={CLUSTERS_PER_INSTANCE} baseline_flat_map_collect=1 candidate_segmented_extend=1 baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}

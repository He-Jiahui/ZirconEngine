use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract,
};

use super::page_parent_pages;

const BENCH_CLUSTER_COUNT: usize = 512;
const CHECKS_PER_SAMPLE: usize = 8;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn hashed_cluster_index_preserves_last_duplicate_cluster_definition() {
    let extract = RenderVirtualGeometryExtract {
        clusters: vec![
            cluster(1, 10, None),
            cluster(1, 30, None),
            cluster(2, 20, Some(1)),
        ],
        ..RenderVirtualGeometryExtract::default()
    };

    assert_eq!(page_parent_pages(&extract), BTreeMap::from([(20, 30)]));
}

#[test]
#[ignore = "release-only extract page-parent index benchmark"]
fn extract_page_parent_index_release_benchmark_evidence() {
    let extract = RenderVirtualGeometryExtract {
        clusters: (0..BENCH_CLUSTER_COUNT as u32)
            .map(|cluster_id| cluster(cluster_id, 10, cluster_id.checked_sub(1)))
            .collect(),
        ..RenderVirtualGeometryExtract::default()
    };
    assert_eq!(
        page_parent_pages(&extract),
        legacy_page_parent_pages(&extract)
    );

    for _ in 0..4 {
        black_box(measure_legacy(&extract));
        black_box(measure_optimized(&extract));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&extract));
            optimized_samples.push(measure_optimized(&extract));
        } else {
            optimized_samples.push(measure_optimized(&extract));
            legacy_samples.push(measure_legacy(&extract));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=extract_page_parent_hash_indexes \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
cluster_count={BENCH_CLUSTER_COUNT} lineage_shape=single_page_chain \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_cluster_index=btree legacy_cycle_index=btree \
optimized_cluster_index=preallocated_hash optimized_cycle_index=hash \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns.saturating_mul(3),
        "hashed extract page-parent indexes must reduce P95 by at least 25%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(extract: &RenderVirtualGeometryExtract) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_page_parent_pages(black_box(extract)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(extract: &RenderVirtualGeometryExtract) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(page_parent_pages(black_box(extract)));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_page_parent_pages(extract: &RenderVirtualGeometryExtract) -> BTreeMap<u32, u32> {
    let clusters_by_id = extract
        .clusters
        .iter()
        .copied()
        .map(|cluster| (cluster.cluster_id, cluster))
        .collect::<BTreeMap<_, _>>();
    let mut page_parent_pages = BTreeMap::new();
    for &cluster in &extract.clusters {
        if page_parent_pages.contains_key(&cluster.page_id) {
            continue;
        }
        if let Some(parent_page_id) = legacy_nearest_parent_page(cluster, &clusters_by_id) {
            page_parent_pages.insert(cluster.page_id, parent_page_id);
        }
    }
    page_parent_pages
}

fn legacy_nearest_parent_page(
    cluster: RenderVirtualGeometryCluster,
    clusters_by_id: &BTreeMap<u32, RenderVirtualGeometryCluster>,
) -> Option<u32> {
    let mut current_parent_cluster_id = cluster.parent_cluster_id;
    let mut visited_cluster_ids = BTreeSet::new();
    while let Some(parent_cluster_id) = current_parent_cluster_id {
        if !visited_cluster_ids.insert(parent_cluster_id) {
            break;
        }
        let parent_cluster = clusters_by_id.get(&parent_cluster_id)?;
        if parent_cluster.page_id != cluster.page_id {
            return Some(parent_cluster.page_id);
        }
        current_parent_cluster_id = parent_cluster.parent_cluster_id;
    }
    None
}

fn cluster(
    cluster_id: u32,
    page_id: u32,
    parent_cluster_id: Option<u32>,
) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
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

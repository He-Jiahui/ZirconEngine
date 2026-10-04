use std::{
    collections::BTreeMap,
    hint::black_box,
    time::{Duration, Instant},
};

use crate::virtual_geometry::{
    VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
    VirtualGeometryPrepareDrawSegment,
};
use zircon_runtime::graphics::VisibilityVirtualGeometryCluster;

use super::compact_cluster_draw_segments;

const CAPACITY_CONTRACT_CLUSTER_COUNT: usize = 65;
const BENCHMARK_SEGMENT_COUNT: usize = 16_385;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn compact_draw_segments_reserve_the_visible_cluster_upper_bound() {
    let visible_clusters = (0..CAPACITY_CONTRACT_CLUSTER_COUNT as u32)
        .map(visible_cluster)
        .collect::<Vec<_>>();
    let prepared_clusters = visible_clusters
        .iter()
        .map(prepared_cluster)
        .collect::<Vec<_>>();
    let prepared_clusters_by_id = prepared_clusters
        .iter()
        .map(|cluster| ((cluster.entity, cluster.cluster_id), cluster))
        .collect::<BTreeMap<_, _>>();

    let segments = compact_cluster_draw_segments(&visible_clusters, &prepared_clusters_by_id);

    assert_eq!(segments.len(), CAPACITY_CONTRACT_CLUSTER_COUNT);
    assert_eq!(segments.capacity(), visible_clusters.len());
    assert_eq!(segments.first().map(|segment| segment.page_id), Some(1_000));
    assert_eq!(
        segments.last().map(|segment| segment.page_id),
        Some(1_000 + CAPACITY_CONTRACT_CLUSTER_COUNT as u32 - 1)
    );
}

#[test]
fn exact_capacity_draw_segment_collection_performance_contract() {
    let source = (0..BENCHMARK_SEGMENT_COUNT as u32)
        .map(draw_segment)
        .collect::<Vec<_>>();
    let legacy = || {
        black_box(collect_segments(&source, None));
    };
    let optimized = || {
        black_box(collect_segments(&source, Some(source.len())));
    };

    legacy();
    optimized();
    let legacy_capacity = collect_segments(&source, None).capacity();
    let optimized_capacity = collect_segments(&source, Some(source.len())).capacity();
    let (legacy_samples, optimized_samples) = paired_samples(legacy, optimized);
    let legacy_p50 = nearest_rank(&legacy_samples, 50).as_nanos();
    let legacy_p95 = nearest_rank(&legacy_samples, 95).as_nanos();
    let optimized_p50 = nearest_rank(&optimized_samples, 50).as_nanos();
    let optimized_p95 = nearest_rank(&optimized_samples, 95).as_nanos();

    println!(
        "PERF_RESULT plugins17_exact_capacity_draw_segments segments={BENCHMARK_SEGMENT_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_final_capacity={legacy_capacity} optimized_final_capacity={optimized_capacity} legacy_growth_allocations_min=2 optimized_growth_allocations=1 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_p50} optimized_ns={optimized_p50}"
    );
    assert!(optimized_capacity < legacy_capacity);
    assert!(
        optimized_p95 <= legacy_p95,
        "exact capacity collection must not regress P95: legacy_p95={legacy_p95}ns optimized_p95={optimized_p95}ns"
    );
}

fn visible_cluster(index: u32) -> VisibilityVirtualGeometryCluster {
    VisibilityVirtualGeometryCluster {
        entity: 7,
        stable_instance_key: 7,
        cluster_id: index,
        page_id: 1_000 + index,
        lod_level: 0,
        cluster_ordinal: index,
        cluster_count: CAPACITY_CONTRACT_CLUSTER_COUNT as u32,
        resident: true,
    }
}

fn prepared_cluster(cluster: &VisibilityVirtualGeometryCluster) -> VirtualGeometryPrepareCluster {
    VirtualGeometryPrepareCluster {
        entity: cluster.entity,
        cluster_id: cluster.cluster_id,
        page_id: cluster.page_id,
        lod_level: cluster.lod_level,
        resident_slot: Some(cluster.cluster_id),
        state: VirtualGeometryPrepareClusterState::Resident,
    }
}

fn draw_segment(index: u32) -> VirtualGeometryPrepareDrawSegment {
    VirtualGeometryPrepareDrawSegment {
        entity: 7,
        cluster_id: index,
        page_id: 1_000 + index,
        resident_slot: Some(index),
        cluster_ordinal: index,
        cluster_span_count: 1,
        cluster_count: BENCHMARK_SEGMENT_COUNT as u32,
        lineage_depth: 0,
        lod_level: 0,
        state: VirtualGeometryPrepareClusterState::Resident,
    }
}

fn collect_segments(
    source: &[VirtualGeometryPrepareDrawSegment],
    capacity: Option<usize>,
) -> Vec<VirtualGeometryPrepareDrawSegment> {
    let mut segments = capacity.map_or_else(Vec::new, Vec::with_capacity);
    for segment in source {
        segments.push(segment.clone());
    }
    segments
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

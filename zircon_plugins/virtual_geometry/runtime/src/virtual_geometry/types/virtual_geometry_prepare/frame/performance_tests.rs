use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

use super::{
    FallbackClusterIndexes, FallbackIndirectCluster, PrepareFrameLookupIndexes,
    VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
    VirtualGeometryPrepareDrawSegment, VirtualGeometryPrepareFrame, VirtualGeometryPreparePage,
    VirtualGeometryPrepareRequest,
};
use zircon_runtime::core::framework::scene::EntityId;

const BENCH_ENTITY_COUNT: usize = 4_096;
const CHECKS_PER_SAMPLE: usize = 16;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn prepare_frame_lookup_indexes_preserve_last_wins_duplicate_semantics() {
    let frame = VirtualGeometryPrepareFrame {
        visible_entities: vec![7, 8, 7],
        visible_clusters: vec![cluster(7, 1, 10), cluster(7, 1, 20)],
        cluster_draw_segments: vec![draw_segment(7, 1)],
        resident_pages: vec![page(30, 3)],
        evictable_pages: vec![page(30, 4)],
        pending_page_requests: vec![
            request(40, 1, None, Some(30)),
            request(40, 2, None, Some(30)),
        ],
        available_slots: Vec::new(),
    };

    let indexes = PrepareFrameLookupIndexes::new(&frame);
    assert_eq!(indexes.visible_entity_indices[&7], 2);
    assert_eq!(indexes.cluster_state[&(7, 1)], (20, None));
    assert_eq!(indexes.request_order_by_page[&40], 2);
    assert_eq!(indexes.request_submission_slot_by_page[&40], Some(4));
    assert_eq!(indexes.explicit_entities, HashSet::from([7]));
}

#[test]
#[ignore = "release-only prepare frame lookup index benchmark"]
fn prepare_frame_lookup_indexes_release_benchmark_evidence() {
    let frame = benchmark_frame();
    assert_eq!(
        PrepareFrameLookupIndexes::new(&frame),
        legacy_prepare_frame_lookup_indexes(&frame)
    );

    let (legacy_samples, optimized_samples) = paired_samples(
        || measure_lookup_legacy(&frame),
        || measure_lookup_optimized(&frame),
    );
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=prepare_frame_lookup_indexes \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
entity_count={BENCH_ENTITY_COUNT} request_count={BENCH_ENTITY_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_request_scans=2 optimized_request_scans=1 optimized_indexes=explicit_preallocated \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(9),
        "single-pass prepare frame lookup indexes must reduce P95 by at least 10%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

#[test]
#[ignore = "release-only fallback cluster index benchmark"]
fn fallback_cluster_indexes_release_benchmark_evidence() {
    let frame = benchmark_frame();
    let lookup_indexes = PrepareFrameLookupIndexes::new(&frame);
    let optimized = FallbackClusterIndexes::new(
        &frame,
        &lookup_indexes.visible_entity_indices,
        &lookup_indexes.explicit_entities,
        &lookup_indexes.request_order_by_page,
        &lookup_indexes.request_submission_slot_by_page,
    );
    let legacy = legacy_fallback_cluster_indexes(&frame, &lookup_indexes);
    assert_eq!(optimized, legacy);

    let (legacy_samples, optimized_samples) = paired_samples(
        || measure_fallback_legacy(&frame, &lookup_indexes),
        || measure_fallback_optimized(&frame, &lookup_indexes),
    );
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=fallback_cluster_indexes \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
entity_count={BENCH_ENTITY_COUNT} cluster_count={BENCH_ENTITY_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_cluster_scans=3 legacy_presence_set=allocated optimized_cluster_scans=2 \
optimized_presence_source=total_count_index optimized_indexes=preallocated \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(9),
        "fallback cluster indexes must reduce P95 by at least 10%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn paired_samples(
    mut measure_legacy: impl FnMut() -> u128,
    mut measure_optimized: impl FnMut() -> u128,
) -> (Vec<u128>, Vec<u128>) {
    for _ in 0..4 {
        black_box(measure_legacy());
        black_box(measure_optimized());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure_lookup_legacy(frame: &VirtualGeometryPrepareFrame) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_prepare_frame_lookup_indexes(black_box(frame)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_lookup_optimized(frame: &VirtualGeometryPrepareFrame) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(PrepareFrameLookupIndexes::new(black_box(frame)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_fallback_legacy(
    frame: &VirtualGeometryPrepareFrame,
    lookup_indexes: &PrepareFrameLookupIndexes,
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_fallback_cluster_indexes(
            black_box(frame),
            black_box(lookup_indexes),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_fallback_optimized(
    frame: &VirtualGeometryPrepareFrame,
    lookup_indexes: &PrepareFrameLookupIndexes,
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(FallbackClusterIndexes::new(
            black_box(frame),
            black_box(&lookup_indexes.visible_entity_indices),
            black_box(&lookup_indexes.explicit_entities),
            black_box(&lookup_indexes.request_order_by_page),
            black_box(&lookup_indexes.request_submission_slot_by_page),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_prepare_frame_lookup_indexes(
    frame: &VirtualGeometryPrepareFrame,
) -> PrepareFrameLookupIndexes {
    let visible_entity_indices = frame
        .visible_entities
        .iter()
        .copied()
        .enumerate()
        .map(|(visible_index, entity)| (entity, visible_index))
        .collect::<HashMap<_, _>>();
    let cluster_state = frame
        .visible_clusters
        .iter()
        .map(|cluster| {
            (
                (cluster.entity, cluster.cluster_id),
                (cluster.page_id, cluster.resident_slot),
            )
        })
        .collect::<HashMap<_, _>>();
    let page_slot = frame
        .resident_pages
        .iter()
        .chain(&frame.evictable_pages)
        .map(|page| (page.page_id, page.slot))
        .collect::<HashMap<_, _>>();
    let request_order_by_page = frame
        .pending_page_requests
        .iter()
        .map(|request| (request.page_id, request.frontier_rank))
        .collect::<HashMap<_, _>>();
    let request_submission_slot_by_page = frame
        .pending_page_requests
        .iter()
        .map(|request| {
            (
                request.page_id,
                request.assigned_slot.or_else(|| {
                    request
                        .recycled_page_id
                        .and_then(|page_id| page_slot.get(&page_id).copied())
                }),
            )
        })
        .collect::<HashMap<_, _>>();
    let explicit_entities = frame
        .cluster_draw_segments
        .iter()
        .map(|draw_segment| draw_segment.entity)
        .collect::<HashSet<_>>();
    PrepareFrameLookupIndexes {
        visible_entity_indices,
        cluster_state,
        request_order_by_page,
        request_submission_slot_by_page,
        explicit_entities,
    }
}

fn legacy_fallback_cluster_indexes(
    frame: &VirtualGeometryPrepareFrame,
    lookup_indexes: &PrepareFrameLookupIndexes,
) -> FallbackClusterIndexes {
    let mut clusters_by_entity = HashMap::<EntityId, Vec<FallbackIndirectCluster>>::new();
    let mut entity_cluster_total_count = HashMap::<EntityId, usize>::new();
    for cluster in &frame.visible_clusters {
        *entity_cluster_total_count
            .entry(cluster.entity)
            .or_default() += 1;
    }
    let clusters_present_by_entity = frame
        .visible_clusters
        .iter()
        .map(|cluster| cluster.entity)
        .collect::<HashSet<_>>();
    let mut entity_cluster_ordinal = HashMap::<EntityId, usize>::new();
    for cluster in &frame.visible_clusters {
        let next_cluster_ordinal = entity_cluster_ordinal.entry(cluster.entity).or_default();
        let cluster_ordinal = *next_cluster_ordinal;
        *next_cluster_ordinal += 1;
        if lookup_indexes.explicit_entities.contains(&cluster.entity)
            || !lookup_indexes
                .visible_entity_indices
                .contains_key(&cluster.entity)
            || matches!(cluster.state, VirtualGeometryPrepareClusterState::Missing)
        {
            continue;
        }
        clusters_by_entity
            .entry(cluster.entity)
            .or_default()
            .push(FallbackIndirectCluster {
                entity_cluster_ordinal: cluster_ordinal,
                entity_cluster_total_count: entity_cluster_total_count[&cluster.entity],
                page_id: cluster.page_id,
                frontier_rank: lookup_indexes
                    .request_order_by_page
                    .get(&cluster.page_id)
                    .copied()
                    .unwrap_or_default(),
                resident_slot: cluster.resident_slot,
                submission_slot: cluster.resident_slot.or_else(|| {
                    lookup_indexes
                        .request_submission_slot_by_page
                        .get(&cluster.page_id)
                        .copied()
                        .flatten()
                }),
                lod_level: cluster.lod_level,
                state: cluster.state,
            });
    }
    black_box(clusters_present_by_entity);
    FallbackClusterIndexes {
        clusters_by_entity,
        entity_cluster_total_count,
    }
}

fn benchmark_frame() -> VirtualGeometryPrepareFrame {
    VirtualGeometryPrepareFrame {
        visible_entities: (0..BENCH_ENTITY_COUNT as u64).collect(),
        visible_clusters: (0..BENCH_ENTITY_COUNT as u64)
            .map(|entity| cluster(entity, entity as u32, entity as u32))
            .collect(),
        cluster_draw_segments: (0..BENCH_ENTITY_COUNT as u64)
            .step_by(4)
            .map(|entity| draw_segment(entity, entity as u32))
            .collect(),
        resident_pages: (0..BENCH_ENTITY_COUNT as u32 / 2)
            .map(|page_id| page(page_id, page_id))
            .collect(),
        evictable_pages: (BENCH_ENTITY_COUNT as u32 / 2..BENCH_ENTITY_COUNT as u32)
            .map(|page_id| page(page_id, page_id + 1_000))
            .collect(),
        pending_page_requests: (0..BENCH_ENTITY_COUNT as u32)
            .map(|page_id| {
                request(
                    page_id,
                    page_id,
                    (page_id % 2 == 0).then_some(page_id + 2_000),
                    (page_id % 2 != 0).then_some(page_id),
                )
            })
            .collect(),
        available_slots: Vec::new(),
    }
}

fn cluster(entity: EntityId, cluster_id: u32, page_id: u32) -> VirtualGeometryPrepareCluster {
    VirtualGeometryPrepareCluster {
        entity,
        cluster_id,
        page_id,
        lod_level: 0,
        resident_slot: None,
        state: if cluster_id % 5 == 0 {
            VirtualGeometryPrepareClusterState::Missing
        } else {
            VirtualGeometryPrepareClusterState::PendingUpload
        },
    }
}

fn draw_segment(entity: EntityId, cluster_id: u32) -> VirtualGeometryPrepareDrawSegment {
    VirtualGeometryPrepareDrawSegment {
        entity,
        cluster_id,
        page_id: cluster_id,
        resident_slot: None,
        cluster_ordinal: 0,
        cluster_span_count: 1,
        cluster_count: 1,
        lineage_depth: 0,
        lod_level: 0,
        state: VirtualGeometryPrepareClusterState::PendingUpload,
    }
}

fn page(page_id: u32, slot: u32) -> VirtualGeometryPreparePage {
    VirtualGeometryPreparePage {
        page_id,
        slot,
        size_bytes: 4_096,
    }
}

fn request(
    page_id: u32,
    frontier_rank: u32,
    assigned_slot: Option<u32>,
    recycled_page_id: Option<u32>,
) -> VirtualGeometryPrepareRequest {
    VirtualGeometryPrepareRequest {
        page_id,
        size_bytes: 4_096,
        generation: 1,
        frontier_rank,
        assigned_slot,
        recycled_page_id,
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

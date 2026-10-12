use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, RenderVirtualGeometryCluster,
    RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryExecutionSegment,
    RenderVirtualGeometryExecutionState, RenderVirtualGeometryExtract,
    RenderVirtualGeometryInstance, RenderVirtualGeometrySelectedCluster,
};
use zircon_runtime::core::math::{Transform, Vec3};

use super::rebuild_selected_clusters_from_execution_segments;

const BENCH_INSTANCE_COUNT: usize = 256;
const CHECKS_PER_SAMPLE: usize = 8;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn snapshot_rebuild_index_preserves_first_instance_and_cluster_for_duplicate_keys() {
    let stable_instance_key = 77_u64 << 16;
    let extract = extract_with_instances(
        vec![cluster(77, 10, 100), cluster(88, 10, 200)],
        vec![
            instance(77, stable_instance_key, 0),
            instance(88, stable_instance_key, 1),
        ],
    );
    let snapshot = snapshot_for(&extract);
    let segment = execution_segment(stable_instance_key, 77, 0);

    assert_eq!(
        rebuild_selected_clusters_from_execution_segments(&snapshot, Some(&extract), &[segment]),
        vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity: 77,
            cluster_id: 10,
            cluster_ordinal: 0,
            page_id: 100,
            lod_level: 0,
            state: RenderVirtualGeometryExecutionState::Resident,
        }]
    );
}

#[test]
#[ignore = "release-only snapshot rebuild index benchmark"]
fn snapshot_rebuild_index_release_benchmark_evidence() {
    let clusters = (0..BENCH_INSTANCE_COUNT)
        .map(|index| cluster(index as u64 + 1, index as u32, index as u32 + 10_000))
        .collect::<Vec<_>>();
    let instances = clusters
        .iter()
        .enumerate()
        .map(|(index, cluster)| {
            instance(
                cluster.entity,
                render_mesh_stable_instance_key(cluster.entity, 0),
                index as u32,
            )
        })
        .collect::<Vec<_>>();
    let extract = extract_with_instances(clusters, instances);
    let snapshot = snapshot_for(&extract);
    let execution_segments = extract
        .instances
        .iter()
        .rev()
        .map(|instance| {
            execution_segment(instance.stable_instance_key_or_legacy(), instance.entity, 0)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rebuild_selected_clusters_from_execution_segments(
            &snapshot,
            Some(&extract),
            &execution_segments,
        ),
        legacy_rebuild(&snapshot, &extract, &execution_segments)
    );

    for _ in 0..4 {
        black_box(measure_legacy(&snapshot, &extract, &execution_segments));
        black_box(measure_optimized(&snapshot, &extract, &execution_segments));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&snapshot, &extract, &execution_segments));
            optimized_samples.push(measure_optimized(&snapshot, &extract, &execution_segments));
        } else {
            optimized_samples.push(measure_optimized(&snapshot, &extract, &execution_segments));
            legacy_samples.push(measure_legacy(&snapshot, &extract, &execution_segments));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=snapshot_rebuild_instance_index \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
instance_count={BENCH_INSTANCE_COUNT} segment_count={BENCH_INSTANCE_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_lookup=segment_and_cluster_linear_scans optimized_lookup=preallocated_hash_indexes \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "indexed snapshot rebuild must reduce P95 by at least 75%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(
    snapshot: &RenderVirtualGeometryDebugSnapshot,
    extract: &RenderVirtualGeometryExtract,
    execution_segments: &[RenderVirtualGeometryExecutionSegment],
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_rebuild(
            black_box(snapshot),
            black_box(extract),
            black_box(execution_segments),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(
    snapshot: &RenderVirtualGeometryDebugSnapshot,
    extract: &RenderVirtualGeometryExtract,
    execution_segments: &[RenderVirtualGeometryExecutionSegment],
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(rebuild_selected_clusters_from_execution_segments(
            black_box(snapshot),
            Some(black_box(extract)),
            black_box(execution_segments),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_rebuild(
    snapshot: &RenderVirtualGeometryDebugSnapshot,
    extract: &RenderVirtualGeometryExtract,
    execution_segments: &[RenderVirtualGeometryExecutionSegment],
) -> Vec<RenderVirtualGeometrySelectedCluster> {
    let mut selected_clusters = Vec::new();
    let mut emitted_clusters = HashSet::<(u64, u32)>::new();

    for segment in execution_segments {
        let stable_instance_key = segment.stable_instance_key_or_legacy();
        let instance_clusters = legacy_clusters_for_key(extract, stable_instance_key);
        let start = usize::try_from(segment.cluster_start_ordinal).unwrap_or(usize::MAX);
        let span = usize::try_from(segment.cluster_span_count).unwrap_or(0);
        let end = start.saturating_add(span).min(instance_clusters.len());
        if start >= end {
            continue;
        }

        for (cluster_ordinal, cluster) in instance_clusters[start..end]
            .iter()
            .enumerate()
            .map(|(index, cluster)| (start.saturating_add(index), cluster))
        {
            if !emitted_clusters.insert((stable_instance_key, cluster.cluster_id)) {
                continue;
            }
            selected_clusters.push(RenderVirtualGeometrySelectedCluster {
                instance_index: segment.instance_index.or_else(|| {
                    legacy_instance_index(
                        snapshot,
                        extract,
                        stable_instance_key,
                        cluster.cluster_id,
                    )
                }),
                entity: cluster.entity,
                cluster_id: cluster.cluster_id,
                cluster_ordinal: u32::try_from(cluster_ordinal).unwrap_or(u32::MAX),
                page_id: cluster.page_id,
                lod_level: cluster.lod_level,
                state: segment.state,
            });
        }
    }

    selected_clusters.sort_by_key(|cluster| {
        (
            cluster.instance_index.unwrap_or(u32::MAX),
            cluster.entity,
            cluster.cluster_ordinal,
            cluster.cluster_id,
            cluster.page_id,
            cluster.lod_level,
        )
    });
    selected_clusters
}

fn legacy_clusters_for_key(
    extract: &RenderVirtualGeometryExtract,
    stable_instance_key: u64,
) -> Vec<RenderVirtualGeometryCluster> {
    let mut clusters = extract
        .instances
        .iter()
        .filter(|instance| instance.stable_instance_key_or_legacy() == stable_instance_key)
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

fn legacy_instance_index(
    snapshot: &RenderVirtualGeometryDebugSnapshot,
    extract: &RenderVirtualGeometryExtract,
    stable_instance_key: u64,
    cluster_id: u32,
) -> Option<u32> {
    if snapshot.instances.is_empty() {
        return None;
    }
    extract
        .instances
        .iter()
        .enumerate()
        .find(|(_, instance)| {
            let start = instance.cluster_offset as usize;
            let end = start.saturating_add(instance.cluster_count as usize);
            instance.stable_instance_key_or_legacy() == stable_instance_key
                && extract
                    .clusters
                    .get(start..end)
                    .into_iter()
                    .flatten()
                    .any(|cluster| cluster.cluster_id == cluster_id)
        })
        .and_then(|(instance_index, _)| u32::try_from(instance_index).ok())
}

fn extract_with_instances(
    clusters: Vec<RenderVirtualGeometryCluster>,
    instances: Vec<RenderVirtualGeometryInstance>,
) -> RenderVirtualGeometryExtract {
    RenderVirtualGeometryExtract {
        cluster_budget: clusters.len() as u32,
        page_budget: 0,
        clusters,
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: Vec::new(),
        page_dependencies: Vec::new(),
        instances,
        debug: Default::default(),
    }
}

fn snapshot_for(extract: &RenderVirtualGeometryExtract) -> RenderVirtualGeometryDebugSnapshot {
    RenderVirtualGeometryDebugSnapshot {
        instances: extract.instances.clone(),
        ..RenderVirtualGeometryDebugSnapshot::default()
    }
}

fn cluster(entity: u64, cluster_id: u32, page_id: u32) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        entity,
        cluster_id,
        hierarchy_node_id: None,
        page_id,
        lod_level: 0,
        parent_cluster_id: None,
        bounds_center: Vec3::ZERO,
        bounds_radius: 0.5,
        screen_space_error: 1.0,
    }
}

fn instance(
    entity: u64,
    stable_instance_key: u64,
    cluster_offset: u32,
) -> RenderVirtualGeometryInstance {
    RenderVirtualGeometryInstance {
        entity,
        stable_instance_key,
        source_model: None,
        transform: Transform::default(),
        cluster_offset,
        cluster_count: 1,
        page_offset: 0,
        page_count: 0,
        mesh_name: None,
        source_hint: None,
    }
}

fn execution_segment(
    stable_instance_key: u64,
    entity: u64,
    cluster_start_ordinal: u32,
) -> RenderVirtualGeometryExecutionSegment {
    RenderVirtualGeometryExecutionSegment {
        original_index: 0,
        instance_index: None,
        entity,
        stable_instance_key,
        page_id: 0,
        draw_ref_index: 0,
        submission_index: Some(0),
        draw_ref_rank: Some(0),
        cluster_start_ordinal,
        cluster_span_count: 1,
        cluster_total_count: 1,
        submission_slot: Some(0),
        state: RenderVirtualGeometryExecutionState::Resident,
        lineage_depth: 0,
        lod_level: 0,
        frontier_rank: 0,
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

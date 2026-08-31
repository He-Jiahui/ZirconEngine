use std::collections::{BTreeSet, HashMap};

use super::support::saturated_u32_len;
use crate::core::framework::render::{
    render_mesh_stable_instance_key, RenderVirtualGeometryCluster,
    RenderVirtualGeometryExecutionSegment, RenderVirtualGeometryExecutionState,
    RenderVirtualGeometryExtract, RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryHardwareRasterizationSource, RenderVirtualGeometryInstance,
    RenderVirtualGeometrySelectedCluster, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometrySubmissionEntry, RenderVirtualGeometrySubmissionRecord,
    RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBuffer64Source,
    RenderVirtualGeometryVisBufferMark,
};
use crate::graphics::VisibilityVirtualGeometryDrawSegment;

pub(super) struct ExecutionSnapshot {
    pub(super) page_ids: BTreeSet<u32>,
    pub(super) resident_segment_count: usize,
    pub(super) pending_segment_count: usize,
    pub(super) missing_segment_count: usize,
    pub(super) repeated_draw_count: usize,
    pub(super) indirect_offsets: Vec<u64>,
    pub(super) segments: Vec<RenderVirtualGeometryExecutionSegment>,
    pub(super) submission_order: Vec<RenderVirtualGeometrySubmissionEntry>,
    pub(super) submission_records: Vec<RenderVirtualGeometrySubmissionRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ExecutionClusterKey {
    entity: u64,
    stable_instance_key: u64,
    cluster_id: u32,
    page_id: u32,
    lod_level: u8,
}

pub(super) struct ExecutionLookup<'a> {
    clusters: &'a [RenderVirtualGeometryCluster],
    instance_indices: Vec<Option<u32>>,
    cluster_ids_by_stable_key: HashMap<u64, Vec<u32>>,
    cluster_indices_by_key: HashMap<ExecutionClusterKey, usize>,
}

impl<'a> ExecutionLookup<'a> {
    pub(super) fn new(extract: &'a RenderVirtualGeometryExtract) -> Self {
        let instance_indices = build_cluster_instance_indices(extract);
        let cluster_ids_by_stable_key = build_cluster_ids_by_stable_key(extract);
        let mut cluster_indices_by_key = HashMap::with_capacity(extract.clusters.len());
        for (cluster_index, cluster) in extract.clusters.iter().enumerate() {
            let stable_instance_key = stable_instance_key_for_cluster_index(
                &extract.instances,
                &instance_indices,
                cluster_index,
                cluster.entity,
            );
            cluster_indices_by_key
                .entry(ExecutionClusterKey {
                    entity: cluster.entity,
                    stable_instance_key,
                    cluster_id: cluster.cluster_id,
                    page_id: cluster.page_id,
                    lod_level: cluster.lod_level,
                })
                .or_insert(cluster_index);
        }

        Self {
            clusters: &extract.clusters,
            instance_indices,
            cluster_ids_by_stable_key,
            cluster_indices_by_key,
        }
    }

    fn instance_index_for_draw_segment(
        &self,
        segment: &VisibilityVirtualGeometryDrawSegment,
    ) -> Option<u32> {
        self.cluster_index(
            segment.entity,
            segment.stable_instance_key,
            segment.cluster_id,
            segment.page_id,
            segment.lod_level,
        )
        .and_then(|cluster_index| self.instance_indices[cluster_index])
    }

    fn cluster_for_execution_ordinal(
        &self,
        segment: &RenderVirtualGeometryExecutionSegment,
        cluster_ordinal: u32,
    ) -> Option<(usize, &'a RenderVirtualGeometryCluster)> {
        let stable_instance_key = segment.stable_instance_key_or_legacy();
        let cluster_id = *self
            .cluster_ids_by_stable_key
            .get(&stable_instance_key)?
            .get(cluster_ordinal as usize)?;
        let cluster_index = self.cluster_index(
            segment.entity,
            stable_instance_key,
            cluster_id,
            segment.page_id,
            segment.lod_level,
        )?;
        Some((cluster_index, &self.clusters[cluster_index]))
    }

    fn cluster_index(
        &self,
        entity: u64,
        stable_instance_key: u64,
        cluster_id: u32,
        page_id: u32,
        lod_level: u8,
    ) -> Option<usize> {
        self.cluster_indices_by_key
            .get(&ExecutionClusterKey {
                entity,
                stable_instance_key,
                cluster_id,
                page_id,
                lod_level,
            })
            .copied()
    }
}

fn build_cluster_instance_indices(extract: &RenderVirtualGeometryExtract) -> Vec<Option<u32>> {
    let mut instance_indices = vec![None; extract.clusters.len()];
    for (instance_index, instance) in extract.instances.iter().enumerate() {
        let Some(instance_index) = u32::try_from(instance_index).ok() else {
            break;
        };
        let start = instance.cluster_offset as usize;
        let end = instance
            .cluster_offset
            .saturating_add(instance.cluster_count) as usize;
        for cluster_instance_index in instance_indices.get_mut(start..end).into_iter().flatten() {
            if cluster_instance_index.is_none() {
                *cluster_instance_index = Some(instance_index);
            }
        }
    }
    instance_indices
}

fn build_cluster_ids_by_stable_key(
    extract: &RenderVirtualGeometryExtract,
) -> HashMap<u64, Vec<u32>> {
    let mut cluster_ids_by_stable_key = HashMap::<u64, Vec<u32>>::new();
    if extract.instances.is_empty() {
        for cluster in &extract.clusters {
            cluster_ids_by_stable_key
                .entry(render_mesh_stable_instance_key(cluster.entity, 0))
                .or_default()
                .push(cluster.cluster_id);
        }
    } else {
        for instance in &extract.instances {
            let start = instance.cluster_offset as usize;
            let end = instance
                .cluster_offset
                .saturating_add(instance.cluster_count) as usize;
            cluster_ids_by_stable_key
                .entry(stable_instance_key_for_instance(instance))
                .or_default()
                .extend(
                    extract
                        .clusters
                        .get(start..end)
                        .into_iter()
                        .flatten()
                        .map(|cluster| cluster.cluster_id),
                );
        }
    }
    for cluster_ids in cluster_ids_by_stable_key.values_mut() {
        cluster_ids.sort_unstable();
        cluster_ids.dedup();
    }
    cluster_ids_by_stable_key
}

fn stable_instance_key_for_cluster_index(
    instances: &[RenderVirtualGeometryInstance],
    instance_indices: &[Option<u32>],
    cluster_index: usize,
    entity: u64,
) -> u64 {
    instance_indices
        .get(cluster_index)
        .copied()
        .flatten()
        .and_then(|instance_index| instances.get(instance_index as usize))
        .map(stable_instance_key_for_instance)
        .unwrap_or_else(|| render_mesh_stable_instance_key(entity, 0))
}

pub(super) fn build_execution_snapshot(
    lookup: &ExecutionLookup<'_>,
    draw_segments: &[VisibilityVirtualGeometryDrawSegment],
    resident_page_set: &BTreeSet<u32>,
    requested_page_set: &BTreeSet<u32>,
) -> ExecutionSnapshot {
    let mut seen_pages = BTreeSet::new();
    let mut page_ids = BTreeSet::new();
    let mut resident_segment_count = 0;
    let mut pending_segment_count = 0;
    let mut missing_segment_count = 0;
    let mut repeated_draw_count = 0;
    let mut indirect_offsets = Vec::with_capacity(draw_segments.len());
    let mut segments = Vec::with_capacity(draw_segments.len());
    let mut submission_order = Vec::with_capacity(draw_segments.len());
    let mut submission_records = Vec::with_capacity(draw_segments.len());

    for (index, segment) in draw_segments.iter().enumerate() {
        let state =
            execution_state_for_page(segment.page_id, resident_page_set, requested_page_set);
        match state {
            RenderVirtualGeometryExecutionState::Resident => resident_segment_count += 1,
            RenderVirtualGeometryExecutionState::PendingUpload => pending_segment_count += 1,
            RenderVirtualGeometryExecutionState::Missing => missing_segment_count += 1,
        }
        if !seen_pages.insert(segment.page_id) {
            repeated_draw_count += 1;
        }
        if state == RenderVirtualGeometryExecutionState::Missing {
            continue;
        }

        let index_u32 = u32::try_from(index).unwrap_or(u32::MAX);
        let submission_index = saturated_u32_len(segments.len());
        let instance_index = lookup.instance_index_for_draw_segment(segment);
        page_ids.insert(segment.page_id);
        indirect_offsets.push(u64::from(submission_index));
        segments.push(RenderVirtualGeometryExecutionSegment {
            original_index: index_u32,
            instance_index,
            entity: segment.entity,
            stable_instance_key: segment.stable_instance_key,
            page_id: segment.page_id,
            draw_ref_index: index_u32,
            submission_index: Some(submission_index),
            draw_ref_rank: Some(submission_index),
            cluster_start_ordinal: segment.cluster_ordinal,
            cluster_span_count: segment.cluster_span_count,
            cluster_total_count: segment.cluster_count,
            submission_slot: Some(submission_index),
            state,
            lineage_depth: segment.lineage_depth,
            lod_level: segment.lod_level,
            frontier_rank: submission_index,
        });
        submission_order.push(RenderVirtualGeometrySubmissionEntry {
            instance_index,
            entity: segment.entity,
            page_id: segment.page_id,
        });
        submission_records.push(RenderVirtualGeometrySubmissionRecord {
            instance_index,
            entity: segment.entity,
            page_id: segment.page_id,
            draw_ref_index: Some(index_u32),
            submission_index,
            draw_ref_rank: submission_index,
            original_index: index_u32,
        });
    }

    ExecutionSnapshot {
        page_ids,
        resident_segment_count,
        pending_segment_count,
        missing_segment_count,
        repeated_draw_count,
        indirect_offsets,
        segments,
        submission_order,
        submission_records,
    }
}

fn execution_state_for_page(
    page_id: u32,
    resident_page_set: &BTreeSet<u32>,
    requested_page_set: &BTreeSet<u32>,
) -> RenderVirtualGeometryExecutionState {
    if resident_page_set.contains(&page_id) {
        RenderVirtualGeometryExecutionState::Resident
    } else if requested_page_set.contains(&page_id) {
        RenderVirtualGeometryExecutionState::PendingUpload
    } else {
        RenderVirtualGeometryExecutionState::Missing
    }
}

fn visbuffer_mark_color(cluster_id: u32, page_id: u32, lod_level: u8) -> [u8; 4] {
    let lod_level = u32::from(lod_level);
    [
        (32 + ((cluster_id * 17 + page_id * 13) % 192)) as u8,
        (32 + ((page_id * 11 + lod_level * 7) % 192)) as u8,
        (32 + ((cluster_id * 5 + lod_level * 19) % 192)) as u8,
        255,
    ]
}

pub(super) fn build_selected_clusters_from_execution_segments(
    lookup: &ExecutionLookup<'_>,
    execution_segments: &[RenderVirtualGeometryExecutionSegment],
) -> Vec<RenderVirtualGeometrySelectedCluster> {
    let mut selected_clusters = Vec::new();

    for segment in execution_segments {
        for ordinal_offset in 0..segment.cluster_span_count {
            let cluster_ordinal = segment.cluster_start_ordinal.saturating_add(ordinal_offset);
            if let Some((cluster_array_index, cluster)) =
                lookup.cluster_for_execution_ordinal(segment, cluster_ordinal)
            {
                selected_clusters.push(RenderVirtualGeometrySelectedCluster {
                    instance_index: lookup.instance_indices[cluster_array_index],
                    entity: cluster.entity,
                    cluster_id: cluster.cluster_id,
                    cluster_ordinal,
                    page_id: cluster.page_id,
                    lod_level: cluster.lod_level,
                    state: segment.state,
                });
            }
        }
    }

    selected_clusters
}

fn stable_instance_key_for_instance(instance: &RenderVirtualGeometryInstance) -> u64 {
    instance.stable_instance_key_or_legacy()
}

pub(super) fn build_visbuffer_debug_marks_from_selected_clusters(
    selected_clusters: &[RenderVirtualGeometrySelectedCluster],
) -> Vec<RenderVirtualGeometryVisBufferMark> {
    selected_clusters
        .iter()
        .map(|cluster| RenderVirtualGeometryVisBufferMark {
            instance_index: cluster.instance_index,
            entity: cluster.entity,
            cluster_id: cluster.cluster_id,
            page_id: cluster.page_id,
            lod_level: cluster.lod_level,
            state: cluster.state,
            color_rgba: visbuffer_mark_color(
                cluster.cluster_id,
                cluster.page_id,
                cluster.lod_level,
            ),
        })
        .collect()
}

pub(super) fn build_visbuffer64_entries_from_selected_clusters(
    selected_clusters: &[RenderVirtualGeometrySelectedCluster],
) -> Vec<RenderVirtualGeometryVisBuffer64Entry> {
    selected_clusters
        .iter()
        .enumerate()
        .map(|(entry_index, cluster)| {
            RenderVirtualGeometryVisBuffer64Entry::from_selected_cluster(
                u32::try_from(entry_index).unwrap_or(u32::MAX),
                cluster,
            )
        })
        .collect()
}

pub(super) fn build_hardware_rasterization_records_from_execution_segments(
    draw_segments: &[VisibilityVirtualGeometryDrawSegment],
    execution: &ExecutionSnapshot,
) -> Vec<RenderVirtualGeometryHardwareRasterizationRecord> {
    let mut records = Vec::with_capacity(execution.segments.len());
    let mut resident_slot = 0_u32;

    for segment in &execution.segments {
        let Some(source_segment) = draw_segments.get(segment.original_index as usize) else {
            continue;
        };
        let resident_slot_for_record =
            (segment.state == RenderVirtualGeometryExecutionState::Resident).then(|| {
                let slot = resident_slot;
                resident_slot = resident_slot.saturating_add(1);
                slot
            });

        records.push(RenderVirtualGeometryHardwareRasterizationRecord {
            instance_index: segment.instance_index,
            entity: segment.entity,
            cluster_id: source_segment.cluster_id,
            cluster_ordinal: segment.cluster_start_ordinal,
            page_id: segment.page_id,
            lod_level: segment.lod_level,
            submission_index: segment
                .submission_index
                .unwrap_or_else(|| saturated_u32_len(records.len())),
            submission_page_id: segment.page_id,
            submission_lod_level: segment.lod_level,
            entity_cluster_start_ordinal: segment.cluster_start_ordinal,
            entity_cluster_span_count: segment.cluster_span_count,
            entity_cluster_total_count: segment.cluster_total_count,
            lineage_depth: segment.lineage_depth,
            frontier_rank: segment.frontier_rank,
            resident_slot: resident_slot_for_record,
            submission_slot: segment.submission_slot,
            state: segment.state,
        });
    }

    records
}

pub(super) fn selected_cluster_source_for_execution(
    has_execution_selections: bool,
) -> RenderVirtualGeometrySelectedClusterSource {
    if has_execution_selections {
        RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections
    } else {
        RenderVirtualGeometrySelectedClusterSource::RenderPathClearOnly
    }
}

pub(super) fn hardware_rasterization_source_for_execution(
    has_execution_selections: bool,
) -> RenderVirtualGeometryHardwareRasterizationSource {
    if has_execution_selections {
        RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections
    } else {
        RenderVirtualGeometryHardwareRasterizationSource::RenderPathClearOnly
    }
}

pub(super) fn visbuffer64_source_for_execution(
    has_execution_selections: bool,
) -> RenderVirtualGeometryVisBuffer64Source {
    if has_execution_selections {
        RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections
    } else {
        RenderVirtualGeometryVisBuffer64Source::RenderPathClearOnly
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn indexed_execution_lookup_preserves_first_match_and_sorted_unique_ordinal() {
        let entity = 42;
        let extract = RenderVirtualGeometryExtract {
            clusters: vec![
                cluster(entity, 9, 3, 1),
                cluster(entity, 5, 3, 1),
                cluster(entity, 5, 3, 1),
            ],
            instances: vec![
                RenderVirtualGeometryInstance {
                    entity,
                    stable_instance_key: 100,
                    cluster_count: 3,
                    ..RenderVirtualGeometryInstance::default()
                },
                RenderVirtualGeometryInstance {
                    entity,
                    stable_instance_key: 200,
                    cluster_count: 1,
                    ..RenderVirtualGeometryInstance::default()
                },
            ],
            ..RenderVirtualGeometryExtract::default()
        };
        let lookup = ExecutionLookup::new(&extract);

        assert_eq!(
            lookup.instance_index_for_draw_segment(&VisibilityVirtualGeometryDrawSegment {
                entity,
                stable_instance_key: 100,
                cluster_id: 5,
                page_id: 3,
                lod_level: 1,
                ..VisibilityVirtualGeometryDrawSegment::default()
            }),
            Some(0)
        );
        assert_eq!(
            lookup.instance_index_for_draw_segment(&VisibilityVirtualGeometryDrawSegment {
                entity,
                stable_instance_key: 200,
                cluster_id: 9,
                page_id: 3,
                lod_level: 1,
                ..VisibilityVirtualGeometryDrawSegment::default()
            }),
            None
        );

        let segment = execution_segment(entity, 100, 3, 1);
        let (cluster_index, selected) = lookup
            .cluster_for_execution_ordinal(&segment, 0)
            .expect("sorted ordinal zero should resolve the first cluster id 5");
        assert_eq!(cluster_index, 1);
        assert_eq!(selected.cluster_id, 5);
        let (cluster_index, selected) = lookup
            .cluster_for_execution_ordinal(&segment, 1)
            .expect("sorted ordinal one should resolve cluster id 9");
        assert_eq!(cluster_index, 0);
        assert_eq!(selected.cluster_id, 9);
    }

    #[test]
    fn indexed_execution_lookup_preserves_legacy_stable_instance_key() {
        let entity = 77;
        let stable_instance_key = render_mesh_stable_instance_key(entity, 0);
        let extract = RenderVirtualGeometryExtract {
            clusters: vec![cluster(entity, 11, 4, 2)],
            instances: vec![RenderVirtualGeometryInstance {
                entity,
                stable_instance_key: 0,
                cluster_count: 1,
                ..RenderVirtualGeometryInstance::default()
            }],
            ..RenderVirtualGeometryExtract::default()
        };
        let lookup = ExecutionLookup::new(&extract);

        assert_eq!(
            lookup.instance_index_for_draw_segment(&VisibilityVirtualGeometryDrawSegment {
                entity,
                stable_instance_key,
                cluster_id: 11,
                page_id: 4,
                lod_level: 2,
                ..VisibilityVirtualGeometryDrawSegment::default()
            }),
            Some(0)
        );
        let legacy_segment = execution_segment(entity, 0, 4, 2);
        let (cluster_index, selected) = lookup
            .cluster_for_execution_ordinal(&legacy_segment, 0)
            .expect("legacy execution key should resolve through the indexed projection");
        assert_eq!(cluster_index, 0);
        assert_eq!(selected.cluster_id, 11);
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn indexed_virtual_geometry_execution_projection_performance_evidence() {
        const INSTANCE_COUNT: u32 = 64;
        const CLUSTERS_PER_INSTANCE: u32 = 16;
        const SAMPLE_COUNT: usize = 21;

        let instances = (0..INSTANCE_COUNT)
            .map(|instance_index| RenderVirtualGeometryInstance {
                entity: u64::from(instance_index) + 50_000,
                stable_instance_key: u64::from(instance_index) + 10_000,
                cluster_offset: instance_index * CLUSTERS_PER_INSTANCE,
                cluster_count: CLUSTERS_PER_INSTANCE,
                ..RenderVirtualGeometryInstance::default()
            })
            .collect::<Vec<_>>();
        let clusters = instances
            .iter()
            .flat_map(|instance| {
                (0..CLUSTERS_PER_INSTANCE).map(move |ordinal| {
                    cluster(
                        instance.entity,
                        ordinal * 3 + instance.stable_instance_key as u32 * 101,
                        instance.stable_instance_key as u32 % 11,
                        (ordinal % 4) as u8,
                    )
                })
            })
            .collect::<Vec<_>>();
        let extract = RenderVirtualGeometryExtract {
            clusters,
            instances,
            ..RenderVirtualGeometryExtract::default()
        };
        let segments = extract
            .clusters
            .iter()
            .enumerate()
            .map(|(cluster_index, cluster)| {
                let instance = &extract.instances[cluster_index / CLUSTERS_PER_INSTANCE as usize];
                VisibilityVirtualGeometryDrawSegment {
                    entity: cluster.entity,
                    stable_instance_key: instance.stable_instance_key,
                    cluster_id: cluster.cluster_id,
                    page_id: cluster.page_id,
                    cluster_ordinal: (cluster_index % CLUSTERS_PER_INSTANCE as usize) as u32,
                    cluster_span_count: 1,
                    cluster_count: CLUSTERS_PER_INSTANCE,
                    lineage_depth: 0,
                    lod_level: cluster.lod_level,
                }
            })
            .rev()
            .collect::<Vec<_>>();

        let expected = legacy_projection(&extract, &segments);
        let actual = indexed_projection(&extract, &segments);
        assert_eq!(actual, expected);

        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut indexed_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample_index in 0..SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                let started = Instant::now();
                black_box(indexed_projection(
                    black_box(&extract),
                    black_box(&segments),
                ));
                indexed_samples.push(started.elapsed().as_nanos());
                let started = Instant::now();
                black_box(legacy_projection(black_box(&extract), black_box(&segments)));
                legacy_samples.push(started.elapsed().as_nanos());
            } else {
                let started = Instant::now();
                black_box(legacy_projection(black_box(&extract), black_box(&segments)));
                legacy_samples.push(started.elapsed().as_nanos());
                let started = Instant::now();
                black_box(indexed_projection(
                    black_box(&extract),
                    black_box(&segments),
                ));
                indexed_samples.push(started.elapsed().as_nanos());
            }
        }
        legacy_samples.sort_unstable();
        indexed_samples.sort_unstable();
        let legacy_p50 = legacy_samples[SAMPLE_COUNT / 2];
        let indexed_p50 = indexed_samples[SAMPLE_COUNT / 2];
        let legacy_p95 = legacy_samples[SAMPLE_COUNT * 95 / 100];
        let indexed_p95 = indexed_samples[SAMPLE_COUNT * 95 / 100];
        println!(
            "RUNTIME09B_INDEXED_VIRTUAL_GEOMETRY_EXECUTION_BENCH_V1 instances={} clusters={} segments={} legacy_p50_ns={} indexed_p50_ns={} legacy_p95_ns={} indexed_p95_ns={} target_ratio_bp=6000",
            extract.instances.len(),
            extract.clusters.len(),
            segments.len(),
            legacy_p50,
            indexed_p50,
            legacy_p95,
            indexed_p95,
        );
        assert!(
            indexed_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
            "indexed projection P95 {indexed_p95} ns exceeded 60% of legacy {legacy_p95} ns"
        );
    }

    fn indexed_projection(
        extract: &RenderVirtualGeometryExtract,
        segments: &[VisibilityVirtualGeometryDrawSegment],
    ) -> Vec<(Option<u32>, Option<u32>)> {
        let lookup = ExecutionLookup::new(extract);
        segments
            .iter()
            .map(|segment| {
                let instance_index = lookup.instance_index_for_draw_segment(segment);
                let execution_segment = execution_segment(
                    segment.entity,
                    segment.stable_instance_key,
                    segment.page_id,
                    segment.lod_level,
                );
                let cluster_index = lookup
                    .cluster_for_execution_ordinal(&execution_segment, segment.cluster_ordinal)
                    .map(|(cluster_index, _)| cluster_index as u32);
                (instance_index, cluster_index)
            })
            .collect()
    }

    fn legacy_projection(
        extract: &RenderVirtualGeometryExtract,
        segments: &[VisibilityVirtualGeometryDrawSegment],
    ) -> Vec<(Option<u32>, Option<u32>)> {
        segments
            .iter()
            .map(|segment| {
                let draw_cluster_index = extract
                    .clusters
                    .iter()
                    .enumerate()
                    .find(|(cluster_index, cluster)| {
                        cluster.entity == segment.entity
                            && legacy_stable_instance_key_for_cluster_index(
                                extract,
                                *cluster_index,
                                cluster.entity,
                            ) == segment.stable_instance_key
                            && cluster.cluster_id == segment.cluster_id
                            && cluster.page_id == segment.page_id
                            && cluster.lod_level == segment.lod_level
                    })
                    .map(|(cluster_index, _)| cluster_index);
                let ordinal_cluster_index = extract
                    .clusters
                    .iter()
                    .enumerate()
                    .find(|(cluster_index, cluster)| {
                        cluster.entity == segment.entity
                            && legacy_stable_instance_key_for_cluster_index(
                                extract,
                                *cluster_index,
                                cluster.entity,
                            ) == segment.stable_instance_key
                            && cluster.page_id == segment.page_id
                            && cluster.lod_level == segment.lod_level
                            && legacy_cluster_ordinal(extract, cluster, segment.stable_instance_key)
                                == segment.cluster_ordinal
                    })
                    .map(|(cluster_index, _)| cluster_index as u32);
                (
                    draw_cluster_index
                        .and_then(|cluster_index| legacy_instance_index(extract, cluster_index)),
                    ordinal_cluster_index,
                )
            })
            .collect()
    }

    fn legacy_instance_index(
        extract: &RenderVirtualGeometryExtract,
        cluster_index: usize,
    ) -> Option<u32> {
        let cluster_index = u32::try_from(cluster_index).ok()?;
        extract
            .instances
            .iter()
            .enumerate()
            .find(|(_, instance)| {
                cluster_index >= instance.cluster_offset
                    && cluster_index
                        < instance
                            .cluster_offset
                            .saturating_add(instance.cluster_count)
            })
            .and_then(|(instance_index, _)| u32::try_from(instance_index).ok())
    }

    fn legacy_stable_instance_key_for_cluster_index(
        extract: &RenderVirtualGeometryExtract,
        cluster_index: usize,
        entity: u64,
    ) -> u64 {
        legacy_instance_index(extract, cluster_index)
            .and_then(|instance_index| extract.instances.get(instance_index as usize))
            .map(stable_instance_key_for_instance)
            .unwrap_or_else(|| render_mesh_stable_instance_key(entity, 0))
    }

    fn legacy_cluster_ordinal(
        extract: &RenderVirtualGeometryExtract,
        cluster: &RenderVirtualGeometryCluster,
        stable_instance_key: u64,
    ) -> u32 {
        let mut cluster_ids = extract
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
                    .map(|candidate| candidate.cluster_id)
            })
            .collect::<Vec<_>>();
        cluster_ids.sort_unstable();
        cluster_ids.dedup();
        cluster_ids
            .iter()
            .position(|cluster_id| *cluster_id == cluster.cluster_id)
            .unwrap_or_default() as u32
    }

    fn cluster(
        entity: u64,
        cluster_id: u32,
        page_id: u32,
        lod_level: u8,
    ) -> RenderVirtualGeometryCluster {
        RenderVirtualGeometryCluster {
            entity,
            cluster_id,
            page_id,
            lod_level,
            ..RenderVirtualGeometryCluster::default()
        }
    }

    fn execution_segment(
        entity: u64,
        stable_instance_key: u64,
        page_id: u32,
        lod_level: u8,
    ) -> RenderVirtualGeometryExecutionSegment {
        RenderVirtualGeometryExecutionSegment {
            original_index: 0,
            instance_index: None,
            entity,
            stable_instance_key,
            page_id,
            draw_ref_index: 0,
            submission_index: Some(0),
            draw_ref_rank: Some(0),
            cluster_start_ordinal: 0,
            cluster_span_count: 1,
            cluster_total_count: 1,
            submission_slot: Some(0),
            state: RenderVirtualGeometryExecutionState::Resident,
            lineage_depth: 0,
            lod_level,
            frontier_rank: 0,
        }
    }
}

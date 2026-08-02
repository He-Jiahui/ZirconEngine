use std::collections::BTreeSet;

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

pub(super) fn build_execution_snapshot(
    extract: &RenderVirtualGeometryExtract,
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
        let instance_index = instance_index_for_draw_segment(extract, segment);
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

fn instance_index_for_draw_segment(
    extract: &RenderVirtualGeometryExtract,
    segment: &VisibilityVirtualGeometryDrawSegment,
) -> Option<u32> {
    extract
        .clusters
        .iter()
        .enumerate()
        .find(|(cluster_array_index, cluster)| {
            cluster.entity == segment.entity
                && stable_instance_key_for_cluster_array_index(
                    &extract.instances,
                    cluster_array_index,
                    cluster.entity,
                ) == segment.stable_instance_key_or_legacy()
                && cluster.cluster_id == segment.cluster_id
                && cluster.page_id == segment.page_id
                && cluster.lod_level == segment.lod_level
        })
        .and_then(|(cluster_array_index, _)| {
            instance_index_for_cluster_array_index(&extract.instances, cluster_array_index)
        })
}

fn instance_index_for_cluster_array_index(
    instances: &[RenderVirtualGeometryInstance],
    cluster_array_index: usize,
) -> Option<u32> {
    let cluster_array_index = u32::try_from(cluster_array_index).ok()?;
    instances
        .iter()
        .enumerate()
        .find(|(_, instance)| {
            cluster_array_index >= instance.cluster_offset
                && cluster_array_index
                    < instance
                        .cluster_offset
                        .saturating_add(instance.cluster_count)
        })
        .and_then(|(instance_index, _)| u32::try_from(instance_index).ok())
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
    extract: &RenderVirtualGeometryExtract,
    execution_segments: &[RenderVirtualGeometryExecutionSegment],
) -> Vec<RenderVirtualGeometrySelectedCluster> {
    let mut selected_clusters = Vec::new();

    for segment in execution_segments {
        for ordinal_offset in 0..segment.cluster_span_count {
            let cluster_ordinal = segment.cluster_start_ordinal.saturating_add(ordinal_offset);
            if let Some((cluster_array_index, cluster)) =
                cluster_for_execution_ordinal(extract, segment, cluster_ordinal)
            {
                selected_clusters.push(RenderVirtualGeometrySelectedCluster {
                    instance_index: instance_index_for_cluster_array_index(
                        &extract.instances,
                        cluster_array_index,
                    ),
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

fn cluster_for_execution_ordinal<'a>(
    extract: &'a RenderVirtualGeometryExtract,
    segment: &RenderVirtualGeometryExecutionSegment,
    cluster_ordinal: u32,
) -> Option<(usize, &'a RenderVirtualGeometryCluster)> {
    extract
        .clusters
        .iter()
        .enumerate()
        .find(|(cluster_array_index, cluster)| {
            cluster.entity == segment.entity
                && stable_instance_key_for_cluster_array_index(
                    &extract.instances,
                    *cluster_array_index,
                    cluster.entity,
                ) == segment.stable_instance_key_or_legacy()
                && cluster.page_id == segment.page_id
                && cluster.lod_level == segment.lod_level
                && cluster_ordinal_for_stable_instance_key(
                    extract,
                    cluster,
                    segment.stable_instance_key_or_legacy(),
                ) == cluster_ordinal
        })
}

fn cluster_ordinal_for_stable_instance_key(
    extract: &RenderVirtualGeometryExtract,
    cluster: &RenderVirtualGeometryCluster,
    stable_instance_key: u64,
) -> u32 {
    let mut cluster_ids = if extract.instances.is_empty() {
        extract
            .clusters
            .iter()
            .filter(|candidate| {
                render_mesh_stable_instance_key(candidate.entity, 0) == stable_instance_key
            })
            .map(|candidate| candidate.cluster_id)
            .collect::<Vec<_>>()
    } else {
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
                    .map(|candidate| candidate.cluster_id)
            })
            .collect::<Vec<_>>()
    };

    cluster_ids.sort_unstable();
    cluster_ids.dedup();
    cluster_ids
        .iter()
        .position(|cluster_id| *cluster_id == cluster.cluster_id)
        .unwrap_or_default() as u32
}

fn stable_instance_key_for_cluster_array_index(
    instances: &[RenderVirtualGeometryInstance],
    cluster_array_index: usize,
    entity: u64,
) -> u64 {
    instance_index_for_cluster_array_index(instances, cluster_array_index)
        .and_then(|instance_index| instances.get(instance_index as usize))
        .map(stable_instance_key_for_instance)
        .unwrap_or_else(|| render_mesh_stable_instance_key(entity, 0))
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

use std::collections::{HashMap, HashSet};

use crate::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryExecutionState,
    RenderVirtualGeometryExtract, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometryVisBufferMark,
};
use crate::core::framework::scene::EntityId;

use super::{
    VirtualGeometryClusterRasterDraw, VirtualGeometryPrepareClusterState,
    VirtualGeometryPrepareFrame,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryClusterSelection {
    pub(crate) submission_index: u32,
    pub(crate) instance_index: Option<u32>,
    pub(crate) entity: EntityId,
    pub(crate) cluster_id: u32,
    pub(crate) cluster_ordinal: u32,
    pub(crate) page_id: u32,
    pub(crate) lod_level: u8,
    pub(crate) submission_page_id: u32,
    pub(crate) submission_lod_level: u8,
    pub(crate) entity_cluster_start_ordinal: usize,
    pub(crate) entity_cluster_span_count: usize,
    pub(crate) entity_cluster_total_count: usize,
    pub(crate) lineage_depth: u32,
    pub(crate) frontier_rank: u32,
    pub(crate) resident_slot: Option<u32>,
    pub(crate) submission_slot: Option<u32>,
    pub(crate) state: VirtualGeometryPrepareClusterState,
}

impl VirtualGeometryClusterSelection {
    pub(crate) fn to_selected_cluster(self) -> RenderVirtualGeometrySelectedCluster {
        RenderVirtualGeometrySelectedCluster {
            instance_index: self.instance_index,
            entity: self.entity,
            cluster_id: self.cluster_id,
            cluster_ordinal: self.cluster_ordinal,
            page_id: self.page_id,
            lod_level: self.lod_level,
            state: map_prepare_cluster_state(self.state),
        }
    }

    pub(crate) fn to_raster_draw(self) -> VirtualGeometryClusterRasterDraw {
        VirtualGeometryClusterRasterDraw {
            submission_index: self.submission_index,
            instance_index: self.instance_index,
            page_id: self.submission_page_id,
            entity_cluster_start_ordinal: self.entity_cluster_start_ordinal,
            entity_cluster_span_count: self.entity_cluster_span_count,
            entity_cluster_total_count: self.entity_cluster_total_count,
            lineage_depth: self.lineage_depth,
            lod_level: self.submission_lod_level,
            frontier_rank: self.frontier_rank,
            resident_slot: self.resident_slot,
            submission_slot: self.submission_slot,
            state: self.state,
        }
    }

    pub(crate) fn to_visbuffer_debug_mark(self) -> RenderVirtualGeometryVisBufferMark {
        RenderVirtualGeometryVisBufferMark {
            instance_index: self.instance_index,
            entity: self.entity,
            cluster_id: self.cluster_id,
            page_id: self.page_id,
            lod_level: self.lod_level,
            state: map_prepare_cluster_state(self.state),
            color_rgba: visbuffer_mark_color(self.cluster_id, self.page_id, self.lod_level),
        }
    }
}

pub(crate) fn build_cluster_selections(
    frame: &VirtualGeometryPrepareFrame,
    extract: &RenderVirtualGeometryExtract,
) -> Vec<VirtualGeometryClusterSelection> {
    let mut selections = build_cluster_selections_from_unified_indirect(frame, extract);
    if selections.is_empty() {
        selections = build_cluster_selections_from_visible_clusters(frame, extract);
    }
    selections.sort_by_key(|selection| {
        (
            selection.instance_index.unwrap_or(u32::MAX),
            selection.entity,
            selection.cluster_ordinal,
            selection.cluster_id,
            selection.page_id,
            selection.lod_level,
            selection.submission_index,
        )
    });
    selections
}

pub(crate) fn cluster_raster_draws_from_selections(
    selections: &[VirtualGeometryClusterSelection],
) -> HashMap<EntityId, Vec<VirtualGeometryClusterRasterDraw>> {
    let mut draws = HashMap::<EntityId, Vec<VirtualGeometryClusterRasterDraw>>::new();
    let mut emitted_submissions = HashSet::<(EntityId, u32)>::new();

    for selection in selections.iter().copied() {
        if !emitted_submissions.insert((selection.entity, selection.submission_index)) {
            continue;
        }

        draws
            .entry(selection.entity)
            .or_default()
            .push(selection.to_raster_draw());
    }

    for entity_draws in draws.values_mut() {
        entity_draws.sort_by_key(|draw| {
            (
                draw.submission_index,
                draw.submission_slot.unwrap_or(u32::MAX),
                draw.frontier_rank,
                draw.entity_cluster_start_ordinal,
                draw.page_id,
                draw.lod_level,
            )
        });
    }

    draws
}

fn build_cluster_selections_from_unified_indirect(
    frame: &VirtualGeometryPrepareFrame,
    extract: &RenderVirtualGeometryExtract,
) -> Vec<VirtualGeometryClusterSelection> {
    let mut selections = Vec::new();
    let mut emitted_clusters = HashSet::<(EntityId, u32)>::new();

    for (submission_index, draw) in frame.unified_indirect_draws().into_iter().enumerate() {
        let entity_clusters = clusters_for_entity_in_overlay_order(extract, draw.entity);
        if entity_clusters.is_empty() {
            continue;
        }

        let start = usize::try_from(draw.cluster_start_ordinal).unwrap_or(usize::MAX);
        let span = usize::try_from(draw.cluster_span_count).unwrap_or(0).max(1);
        let end = start.saturating_add(span).min(entity_clusters.len());
        if start >= end {
            continue;
        }

        for (cluster_ordinal, cluster) in entity_clusters[start..end]
            .iter()
            .enumerate()
            .map(|(index, cluster)| (start.saturating_add(index), cluster))
        {
            if !emitted_clusters.insert((cluster.entity, cluster.cluster_id)) {
                continue;
            }

            selections.push(VirtualGeometryClusterSelection {
                submission_index: u32::try_from(submission_index).unwrap_or(u32::MAX),
                instance_index: overlay_instance_index_for_cluster(
                    extract,
                    cluster.entity,
                    cluster.cluster_id,
                ),
                entity: cluster.entity,
                cluster_id: cluster.cluster_id,
                cluster_ordinal: u32::try_from(cluster_ordinal).unwrap_or(u32::MAX),
                page_id: cluster.page_id,
                lod_level: cluster.lod_level,
                submission_page_id: draw.page_id,
                submission_lod_level: draw.lod_level,
                entity_cluster_start_ordinal: start,
                entity_cluster_span_count: span,
                entity_cluster_total_count: usize::try_from(draw.cluster_total_count)
                    .unwrap_or(entity_clusters.len())
                    .max(1),
                lineage_depth: draw.lineage_depth,
                frontier_rank: draw.frontier_rank,
                resident_slot: draw.resident_slot,
                submission_slot: draw.submission_slot,
                state: draw.state,
            });
        }
    }

    selections
}

fn build_cluster_selections_from_visible_clusters(
    frame: &VirtualGeometryPrepareFrame,
    extract: &RenderVirtualGeometryExtract,
) -> Vec<VirtualGeometryClusterSelection> {
    let page_slot = frame
        .resident_pages
        .iter()
        .chain(frame.evictable_pages.iter())
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
                        .and_then(|recycled_page_id| page_slot.get(&recycled_page_id).copied())
                }),
            )
        })
        .collect::<HashMap<_, _>>();
    let cluster_total_count_by_entity = frame
        .visible_clusters
        .iter()
        .filter(|cluster| !matches!(cluster.state, VirtualGeometryPrepareClusterState::Missing))
        .fold(HashMap::<EntityId, usize>::new(), |mut counts, cluster| {
            *counts.entry(cluster.entity).or_default() += 1;
            counts
        });

    let mut selections = frame
        .visible_clusters
        .iter()
        .filter(|cluster| !matches!(cluster.state, VirtualGeometryPrepareClusterState::Missing))
        .filter_map(|cluster| {
            let cluster_ordinal =
                overlay_cluster_ordinal_for_cluster(extract, cluster.entity, cluster.cluster_id)?;
            Some(VirtualGeometryClusterSelection {
                submission_index: 0,
                instance_index: overlay_instance_index_for_cluster(
                    extract,
                    cluster.entity,
                    cluster.cluster_id,
                ),
                entity: cluster.entity,
                cluster_id: cluster.cluster_id,
                cluster_ordinal,
                page_id: cluster.page_id,
                lod_level: cluster.lod_level,
                submission_page_id: cluster.page_id,
                submission_lod_level: cluster.lod_level,
                entity_cluster_start_ordinal: usize::try_from(cluster_ordinal).unwrap_or(0),
                entity_cluster_span_count: 1,
                entity_cluster_total_count: cluster_total_count_by_entity
                    .get(&cluster.entity)
                    .copied()
                    .unwrap_or(1)
                    .max(1),
                lineage_depth: 0,
                frontier_rank: request_order_by_page
                    .get(&cluster.page_id)
                    .copied()
                    .unwrap_or_default(),
                resident_slot: cluster.resident_slot,
                submission_slot: cluster.resident_slot.or_else(|| {
                    request_submission_slot_by_page
                        .get(&cluster.page_id)
                        .copied()
                        .flatten()
                }),
                state: cluster.state,
            })
        })
        .collect::<Vec<_>>();

    selections.sort_by_key(|selection| {
        (
            selection.instance_index.unwrap_or(u32::MAX),
            selection.entity,
            selection.cluster_ordinal,
            selection.cluster_id,
            selection.page_id,
            selection.lod_level,
        )
    });

    for (submission_index, selection) in selections.iter_mut().enumerate() {
        selection.submission_index = u32::try_from(submission_index).unwrap_or(u32::MAX);
    }

    selections
}

fn clusters_for_entity_in_overlay_order(
    extract: &RenderVirtualGeometryExtract,
    entity: EntityId,
) -> Vec<RenderVirtualGeometryCluster> {
    let mut clusters = if extract.instances.is_empty() {
        extract
            .clusters
            .iter()
            .copied()
            .filter(|cluster| cluster.entity == entity)
            .collect::<Vec<_>>()
    } else {
        extract
            .instances
            .iter()
            .filter(|instance| instance.entity == entity)
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
            .collect::<Vec<_>>()
    };
    clusters.sort_by_key(|cluster| cluster.cluster_id);
    clusters.dedup_by_key(|cluster| cluster.cluster_id);
    clusters
}

fn overlay_instance_index_for_cluster(
    extract: &RenderVirtualGeometryExtract,
    entity: EntityId,
    cluster_id: u32,
) -> Option<u32> {
    extract
        .instances
        .iter()
        .enumerate()
        .find(|(_, instance)| {
            if instance.entity != entity {
                return false;
            }

            let start = instance.cluster_offset as usize;
            let end = start.saturating_add(instance.cluster_count as usize);
            extract
                .clusters
                .get(start..end)
                .into_iter()
                .flatten()
                .any(|cluster| cluster.cluster_id == cluster_id)
        })
        .and_then(|(instance_index, _)| u32::try_from(instance_index).ok())
}

fn overlay_cluster_ordinal_for_cluster(
    extract: &RenderVirtualGeometryExtract,
    entity: EntityId,
    cluster_id: u32,
) -> Option<u32> {
    clusters_for_entity_in_overlay_order(extract, entity)
        .iter()
        .enumerate()
        .find(|(_, cluster)| cluster.cluster_id == cluster_id)
        .and_then(|(cluster_ordinal, _)| u32::try_from(cluster_ordinal).ok())
}

fn map_prepare_cluster_state(
    state: VirtualGeometryPrepareClusterState,
) -> RenderVirtualGeometryExecutionState {
    match state {
        VirtualGeometryPrepareClusterState::Resident => {
            RenderVirtualGeometryExecutionState::Resident
        }
        VirtualGeometryPrepareClusterState::PendingUpload => {
            RenderVirtualGeometryExecutionState::PendingUpload
        }
        VirtualGeometryPrepareClusterState::Missing => RenderVirtualGeometryExecutionState::Missing,
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

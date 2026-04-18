use std::collections::HashMap;

use zircon_scene::EntityId;

use super::{
    VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
    VirtualGeometryPrepareDrawSegment, VirtualGeometryPrepareIndirectDraw,
    VirtualGeometryPreparePage, VirtualGeometryPrepareRequest,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryPrepareFrame {
    pub(crate) visible_entities: Vec<EntityId>,
    pub(crate) visible_clusters: Vec<VirtualGeometryPrepareCluster>,
    pub(crate) cluster_draw_segments: Vec<VirtualGeometryPrepareDrawSegment>,
    pub(crate) resident_pages: Vec<VirtualGeometryPreparePage>,
    pub(crate) pending_page_requests: Vec<VirtualGeometryPrepareRequest>,
    pub(crate) available_slots: Vec<u32>,
    pub(crate) evictable_pages: Vec<VirtualGeometryPreparePage>,
}

impl VirtualGeometryPrepareFrame {
    pub(crate) fn unified_indirect_draws(&self) -> Vec<VirtualGeometryPrepareIndirectDraw> {
        let cluster_state = self
            .visible_clusters
            .iter()
            .map(|cluster| {
                (
                    (cluster.entity, cluster.cluster_id),
                    (cluster.page_id, cluster.resident_slot),
                )
            })
            .collect::<HashMap<_, _>>();
        let page_slot = self
            .resident_pages
            .iter()
            .chain(self.evictable_pages.iter())
            .map(|page| (page.page_id, page.slot))
            .collect::<HashMap<_, _>>();
        let request_order_by_page = self
            .pending_page_requests
            .iter()
            .map(|request| (request.page_id, request.frontier_rank))
            .collect::<HashMap<_, _>>();
        let request_submission_slot_by_page = self
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
        let mut indirect_draws = self
            .cluster_draw_segments
            .iter()
            .enumerate()
            .filter(|draw_segment| {
                !matches!(
                    draw_segment.1.state,
                    VirtualGeometryPrepareClusterState::Missing
                )
            })
            .map(|(original_index, draw_segment)| {
                let cluster_state = cluster_state
                    .get(&(draw_segment.entity, draw_segment.cluster_id))
                    .copied();
                let page_id = if draw_segment.page_id != 0 {
                    draw_segment.page_id
                } else {
                    cluster_state
                        .map(|(page_id, _resident_slot)| page_id)
                        .unwrap_or_default()
                };
                let resident_slot = draw_segment
                    .resident_slot
                    .or_else(|| cluster_state.and_then(|(_page_id, resident_slot)| resident_slot));
                let submission_slot = resident_slot.or_else(|| {
                    request_submission_slot_by_page
                        .get(&page_id)
                        .copied()
                        .flatten()
                });
                (
                    original_index,
                    VirtualGeometryPrepareIndirectDraw {
                        entity: draw_segment.entity,
                        page_id,
                        cluster_start_ordinal: draw_segment.cluster_ordinal,
                        cluster_span_count: draw_segment.cluster_span_count.max(1),
                        cluster_total_count: draw_segment.cluster_count.max(1),
                        lineage_depth: draw_segment.lineage_depth,
                        lod_level: draw_segment.lod_level,
                        frontier_rank: request_order_by_page
                            .get(&page_id)
                            .copied()
                            .unwrap_or_default(),
                        resident_slot,
                        submission_slot,
                        state: draw_segment.state,
                    },
                )
            })
            .collect::<Vec<_>>();
        indirect_draws.sort_by_key(|(original_index, draw)| {
            (
                draw.submission_slot.unwrap_or(u32::MAX),
                draw.frontier_rank,
                draw.entity,
                draw.cluster_start_ordinal,
                draw.page_id,
                draw.cluster_span_count,
                draw.cluster_total_count,
                draw.lod_level,
                draw.lineage_depth,
                encode_cluster_state(draw.state),
                *original_index,
            )
        });
        indirect_draws
            .into_iter()
            .map(|(_original_index, draw)| draw)
            .collect()
    }
}

fn encode_cluster_state(state: VirtualGeometryPrepareClusterState) -> u32 {
    match state {
        VirtualGeometryPrepareClusterState::Resident => 0,
        VirtualGeometryPrepareClusterState::PendingUpload => 1,
        VirtualGeometryPrepareClusterState::Missing => 2,
    }
}

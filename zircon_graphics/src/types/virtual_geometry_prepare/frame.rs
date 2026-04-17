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
        self.cluster_draw_segments
            .iter()
            .filter(|draw_segment| {
                !matches!(
                    draw_segment.state,
                    VirtualGeometryPrepareClusterState::Missing
                )
            })
            .map(|draw_segment| {
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
                VirtualGeometryPrepareIndirectDraw {
                    entity: draw_segment.entity,
                    page_id,
                    cluster_start_ordinal: draw_segment.cluster_ordinal,
                    cluster_span_count: draw_segment.cluster_span_count.max(1),
                    cluster_total_count: draw_segment.cluster_count.max(1),
                    lod_level: draw_segment.lod_level,
                    resident_slot,
                    state: draw_segment.state,
                }
            })
            .collect()
    }
}

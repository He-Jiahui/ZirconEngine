use crate::core::framework::scene::EntityId;

use super::VirtualGeometryPrepareClusterState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryPrepareIndirectDraw {
    pub(crate) entity: EntityId,
    pub(crate) page_id: u32,
    pub(crate) cluster_start_ordinal: u32,
    pub(crate) cluster_span_count: u32,
    pub(crate) cluster_total_count: u32,
    pub(crate) lineage_depth: u32,
    pub(crate) lod_level: u8,
    pub(crate) frontier_rank: u32,
    pub(crate) resident_slot: Option<u32>,
    pub(crate) submission_slot: Option<u32>,
    pub(crate) state: VirtualGeometryPrepareClusterState,
}

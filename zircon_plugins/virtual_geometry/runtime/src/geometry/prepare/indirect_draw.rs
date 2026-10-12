use zircon_runtime::core::framework::scene::EntityId;

use super::VirtualGeometryPrepareClusterState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VirtualGeometryPrepareIndirectDraw {
    pub entity: EntityId,
    pub page_id: u32,
    pub cluster_start_ordinal: u32,
    pub cluster_span_count: u32,
    pub cluster_total_count: u32,
    pub lineage_depth: u32,
    pub lod_level: u8,
    pub frontier_rank: u32,
    pub resident_slot: Option<u32>,
    pub submission_slot: Option<u32>,
    pub state: VirtualGeometryPrepareClusterState,
}

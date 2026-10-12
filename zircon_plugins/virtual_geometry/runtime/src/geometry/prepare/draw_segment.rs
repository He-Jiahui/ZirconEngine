use zircon_runtime::core::framework::scene::EntityId;

use super::VirtualGeometryPrepareClusterState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VirtualGeometryPrepareDrawSegment {
    pub entity: EntityId,
    pub cluster_id: u32,
    pub page_id: u32,
    pub resident_slot: Option<u32>,
    pub cluster_ordinal: u32,
    pub cluster_span_count: u32,
    pub cluster_count: u32,
    pub lineage_depth: u32,
    pub lod_level: u8,
    pub state: VirtualGeometryPrepareClusterState,
}

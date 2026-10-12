use zircon_runtime::core::framework::scene::EntityId;

use super::VirtualGeometryPrepareClusterState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VirtualGeometryPrepareCluster {
    pub entity: EntityId,
    pub cluster_id: u32,
    pub page_id: u32,
    pub lod_level: u8,
    pub resident_slot: Option<u32>,
    pub state: VirtualGeometryPrepareClusterState,
}

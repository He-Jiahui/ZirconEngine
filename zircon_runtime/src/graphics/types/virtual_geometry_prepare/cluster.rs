use crate::core::framework::scene::EntityId;

use super::VirtualGeometryPrepareClusterState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryPrepareCluster {
    pub(crate) entity: EntityId,
    pub(crate) cluster_id: u32,
    pub(crate) page_id: u32,
    pub(crate) lod_level: u8,
    pub(crate) resident_slot: Option<u32>,
    pub(crate) state: VirtualGeometryPrepareClusterState,
}

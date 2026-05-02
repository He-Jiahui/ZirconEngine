use super::VirtualGeometryPrepareClusterState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryClusterRasterDraw {
    pub(crate) submission_index: u32,
    pub(crate) instance_index: Option<u32>,
    #[allow(dead_code)]
    pub(crate) page_id: u32,
    pub(crate) entity_cluster_start_ordinal: usize,
    pub(crate) entity_cluster_span_count: usize,
    pub(crate) entity_cluster_total_count: usize,
    pub(crate) lineage_depth: u32,
    pub(crate) lod_level: u8,
    pub(crate) frontier_rank: u32,
    pub(crate) resident_slot: Option<u32>,
    pub(crate) submission_slot: Option<u32>,
    pub(crate) state: VirtualGeometryPrepareClusterState,
}

use crate::types::VirtualGeometryPrepareClusterState;

#[derive(Clone, Copy, Debug)]
pub(super) struct VirtualGeometryClusterRasterDraw {
    pub(super) submission_index: u32,
    #[allow(dead_code)]
    pub(super) page_id: u32,
    pub(super) entity_cluster_start_ordinal: usize,
    pub(super) entity_cluster_span_count: usize,
    pub(super) entity_cluster_total_count: usize,
    pub(super) lineage_depth: u32,
    pub(super) lod_level: u8,
    pub(super) frontier_rank: u32,
    pub(super) resident_slot: Option<u32>,
    pub(super) submission_slot: Option<u32>,
    pub(super) state: VirtualGeometryPrepareClusterState,
}

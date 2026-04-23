use super::*;

pub(in super::super) fn selection(
    instance_index: Option<u32>,
    entity: u64,
    submission_index: u32,
    cluster_id: u32,
    cluster_ordinal: u32,
    page_id: u32,
    lod_level: u8,
    state: VirtualGeometryPrepareClusterState,
) -> VirtualGeometryClusterSelection {
    VirtualGeometryClusterSelection {
        submission_index,
        instance_index,
        entity,
        cluster_id,
        cluster_ordinal,
        page_id,
        lod_level,
        submission_page_id: page_id,
        submission_lod_level: lod_level,
        entity_cluster_start_ordinal: cluster_ordinal as usize,
        entity_cluster_span_count: 1,
        entity_cluster_total_count: 3,
        lineage_depth: 0,
        frontier_rank: 0,
        resident_slot: Some(0),
        submission_slot: Some(0),
        state,
    }
}

pub(in super::super) fn clusters_by_id(
    extract: &RenderVirtualGeometryExtract,
) -> HashMap<u32, RenderVirtualGeometryCluster> {
    extract
        .clusters
        .iter()
        .copied()
        .map(|cluster| (cluster.cluster_id, cluster))
        .collect()
}

pub(in super::super) fn cluster_ordering(
    extract: &RenderVirtualGeometryExtract,
) -> HashMap<(u64, u32), SeedBackedClusterOrdering> {
    seed_backed_cluster_ordering(extract)
}

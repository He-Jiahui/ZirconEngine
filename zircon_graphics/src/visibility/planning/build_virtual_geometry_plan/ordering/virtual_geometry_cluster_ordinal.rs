use zircon_framework::render::{RenderVirtualGeometryCluster, RenderVirtualGeometryExtract};

pub(in crate::visibility::planning::build_virtual_geometry_plan) fn virtual_geometry_cluster_ordinal(
    extract: &RenderVirtualGeometryExtract,
    cluster: &RenderVirtualGeometryCluster,
) -> u32 {
    let mut cluster_ids = extract
        .clusters
        .iter()
        .filter(|candidate| candidate.entity == cluster.entity)
        .map(|candidate| candidate.cluster_id)
        .collect::<Vec<_>>();
    cluster_ids.sort_unstable();
    cluster_ids.dedup();
    cluster_ids
        .iter()
        .position(|cluster_id| *cluster_id == cluster.cluster_id)
        .unwrap_or_default() as u32
}

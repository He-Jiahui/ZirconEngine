use crate::core::framework::render::{RenderVirtualGeometryCluster, RenderVirtualGeometryExtract};

use super::cluster_ids_for_entity;

pub(in crate::graphics::visibility::planning::build_virtual_geometry_plan) fn virtual_geometry_cluster_ordinal(
    extract: &RenderVirtualGeometryExtract,
    cluster: &RenderVirtualGeometryCluster,
) -> u32 {
    let cluster_ids = cluster_ids_for_entity(extract, cluster.entity);
    cluster_ids
        .iter()
        .position(|cluster_id| *cluster_id == cluster.cluster_id)
        .unwrap_or_default() as u32
}

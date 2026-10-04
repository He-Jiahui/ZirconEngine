use crate::core::framework::render::{RenderVirtualGeometryCluster, RenderVirtualGeometryExtract};

use super::cluster_ids_for_stable_instance_key;

// 返回 cluster 在该稳定实例的去重排序 ID 表中的序号；调用方须使用同一抽取快照与实例键。
pub(in crate::graphics::visibility::planning::build_virtual_geometry_plan) fn virtual_geometry_cluster_ordinal(
    extract: &RenderVirtualGeometryExtract,
    cluster: &RenderVirtualGeometryCluster,
    stable_instance_key: u64,
) -> u32 {
    let cluster_ids = cluster_ids_for_stable_instance_key(extract, stable_instance_key);
    cluster_ids
        .binary_search(&cluster.cluster_id)
        .unwrap_or_default() as u32
}

#[cfg(test)]
#[path = "tests/virtual_geometry_cluster_ordinal.rs"]
mod tests;

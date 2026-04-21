use crate::core::framework::render::RenderVirtualGeometryExtract;

use super::cluster_ids_for_entity;

pub(in crate::graphics::visibility::planning::build_virtual_geometry_plan) fn virtual_geometry_cluster_count(
    extract: &RenderVirtualGeometryExtract,
    entity: u64,
) -> u32 {
    cluster_ids_for_entity(extract, entity).len().max(1) as u32
}

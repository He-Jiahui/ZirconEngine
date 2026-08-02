use crate::core::framework::render::RenderVirtualGeometryExtract;

use super::cluster_ids_for_stable_instance_key;

pub(in crate::graphics::visibility::planning::build_virtual_geometry_plan) fn virtual_geometry_cluster_count(
    extract: &RenderVirtualGeometryExtract,
    stable_instance_key: u64,
) -> u32 {
    cluster_ids_for_stable_instance_key(extract, stable_instance_key)
        .len()
        .max(1) as u32
}

use std::cmp::Ordering;

use crate::core::framework::render::RenderVirtualGeometryCluster;

pub(in crate::graphics::visibility::planning::build_virtual_geometry_plan) fn virtual_geometry_cluster_sort_key(
    left: &RenderVirtualGeometryCluster,
    right: &RenderVirtualGeometryCluster,
) -> Ordering {
    right
        .screen_space_error
        .partial_cmp(&left.screen_space_error)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.lod_level.cmp(&right.lod_level))
        .then_with(|| left.cluster_id.cmp(&right.cluster_id))
}

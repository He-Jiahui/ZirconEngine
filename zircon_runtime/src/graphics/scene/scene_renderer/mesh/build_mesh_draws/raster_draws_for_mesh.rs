use crate::core::math::Vec4;

use super::virtual_geometry_cluster_raster_draw::VirtualGeometryClusterRasterDraw;
use super::virtual_geometry_cluster_streaming_tint::virtual_geometry_cluster_streaming_tint;
use super::virtual_geometry_draw_range::virtual_geometry_draw_range;

pub(super) fn raster_draws_for_mesh(
    mesh_index_count: u32,
    cluster_draws: Option<&[VirtualGeometryClusterRasterDraw]>,
    base_tint: Vec4,
) -> Vec<(u32, u32, Vec4)> {
    cluster_draws
        .map(|cluster_draws| {
            cluster_draws
                .iter()
                .filter_map(|cluster_draw| {
                    let (first_index, draw_index_count) =
                        virtual_geometry_draw_range(mesh_index_count, *cluster_draw);
                    (draw_index_count > 0).then_some((
                        first_index,
                        draw_index_count,
                        base_tint * virtual_geometry_cluster_streaming_tint(*cluster_draw),
                    ))
                })
                .collect()
        })
        .unwrap_or_else(|| vec![(0, mesh_index_count, base_tint)])
}

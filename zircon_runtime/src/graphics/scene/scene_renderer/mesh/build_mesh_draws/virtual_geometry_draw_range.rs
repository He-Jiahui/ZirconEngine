use crate::graphics::types::VirtualGeometryPrepareClusterState;

use super::virtual_geometry_cluster_raster_draw::VirtualGeometryClusterRasterDraw;

pub(super) fn virtual_geometry_draw_range(
    mesh_index_count: u32,
    draw: VirtualGeometryClusterRasterDraw,
) -> (u32, u32) {
    let triangle_count = mesh_index_count / 3;
    if triangle_count == 0 {
        return (0, mesh_index_count);
    }

    let segment_count = draw
        .entity_cluster_total_count
        .min(triangle_count as usize)
        .max(1);
    let segment_ordinal = draw.entity_cluster_start_ordinal % segment_count;
    let start_triangle = ((triangle_count as usize) * segment_ordinal / segment_count) as u32;
    let end_segment_ordinal = (segment_ordinal + draw.entity_cluster_span_count).min(segment_count);
    let mut end_triangle = ((triangle_count as usize) * end_segment_ordinal / segment_count) as u32;
    if end_triangle <= start_triangle {
        end_triangle = (start_triangle + 1).min(triangle_count);
    }
    let segment_triangle_count = end_triangle.saturating_sub(start_triangle);
    if segment_triangle_count == 0 {
        return (start_triangle * 3, 0);
    }

    let visible_triangle_count = match draw.state {
        VirtualGeometryPrepareClusterState::Resident => segment_triangle_count,
        VirtualGeometryPrepareClusterState::PendingUpload => {
            ((segment_triangle_count as f32) * 0.45).ceil() as u32
        }
        VirtualGeometryPrepareClusterState::Missing => 0,
    }
    .clamp(1, segment_triangle_count);

    (start_triangle * 3, visible_triangle_count * 3)
}

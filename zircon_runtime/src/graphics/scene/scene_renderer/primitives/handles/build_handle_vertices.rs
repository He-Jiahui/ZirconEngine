use crate::core::framework::render::HandleElementExtract;

use crate::graphics::scene::scene_renderer::primitives::LineVertex;
use crate::graphics::types::ViewportRenderFrame;

use super::super::line_geometry::{
    ARROW_HEAD_VERTEX_CAPACITY, CROSS_VERTEX_CAPACITY, RING_VERTEX_CAPACITY, append_arrow_head,
    append_cross, append_ring,
};

const LINE_VERTEX_CAPACITY: usize = 2;

fn handle_element_vertex_capacity(element: &HandleElementExtract) -> usize {
    match element {
        HandleElementExtract::AxisLine { .. } => {
            LINE_VERTEX_CAPACITY.saturating_add(ARROW_HEAD_VERTEX_CAPACITY)
        }
        HandleElementExtract::AxisRing { .. } => RING_VERTEX_CAPACITY,
        HandleElementExtract::AxisScale { .. } => {
            LINE_VERTEX_CAPACITY.saturating_add(CROSS_VERTEX_CAPACITY)
        }
        HandleElementExtract::CenterAnchor { .. } => CROSS_VERTEX_CAPACITY,
    }
}

pub(crate) fn build_handle_vertices(frame: &ViewportRenderFrame) -> Vec<LineVertex> {
    let vertex_capacity = frame
        .overlays()
        .handles
        .iter()
        .flat_map(|handle| handle.elements.iter())
        .map(handle_element_vertex_capacity)
        .fold(0usize, usize::saturating_add);
    let mut vertices = Vec::with_capacity(vertex_capacity);
    let camera = frame.effective_camera();
    for handle in &frame.overlays().handles {
        for element in &handle.elements {
            match element {
                HandleElementExtract::AxisLine {
                    start, end, color, ..
                } => {
                    vertices.push(LineVertex::new(*start, *color));
                    vertices.push(LineVertex::new(*end, *color));
                    append_arrow_head(&mut vertices, *start, *end, *color);
                }
                HandleElementExtract::AxisRing {
                    center,
                    normal,
                    radius,
                    color,
                    ..
                } => append_ring(&mut vertices, *center, *normal, *radius, *color),
                HandleElementExtract::AxisScale {
                    start,
                    end,
                    color,
                    handle_size,
                    ..
                } => {
                    vertices.push(LineVertex::new(*start, *color));
                    vertices.push(LineVertex::new(*end, *color));
                    append_cross(
                        &mut vertices,
                        *end,
                        *handle_size,
                        *color,
                        camera.transform.right(),
                        camera.transform.up(),
                    );
                }
                HandleElementExtract::CenterAnchor {
                    position,
                    size,
                    color,
                } => append_cross(
                    &mut vertices,
                    *position,
                    *size,
                    *color,
                    camera.transform.right(),
                    camera.transform.up(),
                ),
            }
        }
    }
    vertices
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{HandleElementExtract, OverlayAxis};
    use crate::core::math::{Vec3, Vec4};

    use super::handle_element_vertex_capacity;

    #[test]
    fn handle_capacity_matches_non_degenerate_element_topology() {
        let elements = [
            HandleElementExtract::AxisLine {
                axis: OverlayAxis::X,
                start: Vec3::ZERO,
                end: Vec3::X,
                color: Vec4::ONE,
                pick_radius: 1.0,
            },
            HandleElementExtract::AxisRing {
                axis: OverlayAxis::Y,
                center: Vec3::ZERO,
                normal: Vec3::Y,
                radius: 1.0,
                color: Vec4::ONE,
                pick_radius: 1.0,
            },
            HandleElementExtract::AxisScale {
                axis: OverlayAxis::Z,
                start: Vec3::ZERO,
                end: Vec3::Z,
                color: Vec4::ONE,
                pick_radius: 1.0,
                handle_size: 1.0,
            },
            HandleElementExtract::CenterAnchor {
                position: Vec3::ZERO,
                size: 1.0,
                color: Vec4::ONE,
            },
        ];

        let capacities = elements.map(|element| handle_element_vertex_capacity(&element));

        assert_eq!(capacities, [6, 96, 6, 4]);
    }
}

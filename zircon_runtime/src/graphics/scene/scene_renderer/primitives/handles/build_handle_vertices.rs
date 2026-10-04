use crate::core::framework::render::HandleElementExtract;

use crate::graphics::scene::scene_renderer::primitives::LineVertex;
use crate::graphics::types::ViewportRenderFrame;

use super::super::line_geometry::{
    append_arrow_head, append_cross, append_ring, ARROW_HEAD_VERTEX_CAPACITY,
    CROSS_VERTEX_CAPACITY, RING_VERTEX_CAPACITY,
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
    // 缩放杆和中心锚点使用有效相机的屏幕基底；轴线与圆环仍沿提取出的世界空间方向。
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
#[path = "tests/build_handle_vertices.rs"]
mod tests;

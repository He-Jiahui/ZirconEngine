use zircon_framework::render::HandleElementExtract;

use crate::scene::scene_renderer::primitives::LineVertex;
use crate::types::EditorOrRuntimeFrame;

use super::super::line_geometry::{append_arrow_head, append_cross, append_ring};

pub(crate) fn build_handle_vertices(frame: &EditorOrRuntimeFrame) -> Vec<LineVertex> {
    let mut vertices = Vec::new();
    let camera = &frame.scene.scene.camera;
    for handle in &frame.scene.overlays.handles {
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

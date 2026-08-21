use crate::core::math::Vec4;

use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::primitives::LineVertex;
use crate::graphics::types::ViewportRenderFrame;

use super::super::line_geometry::{append_bounding_box_vertices, append_cross};

pub(crate) fn build_selection_vertices(
    frame: &ViewportRenderFrame,
    streamer: &ResourceStreamer,
) -> Vec<LineVertex> {
    let mut vertices = Vec::new();
    if let Some(highlights) = frame.overlays().highlights.as_ref() {
        if highlights.attributes().outline_enabled {
            let tint = highlights.attributes().tint_rgba;
            let color = Vec4::new(tint[0], tint[1], tint[2], tint[3]);
            for owner in highlights.entities() {
                let Some(mesh_instance) = frame.meshes().iter().find(|mesh| mesh.node_id == *owner)
                else {
                    continue;
                };
                let Some(model) = streamer.model(&mesh_instance.model.id()) else {
                    continue;
                };
                for mesh in &model.meshes {
                    append_bounding_box_vertices(
                        &mut vertices,
                        mesh.bounds_min,
                        mesh.bounds_max,
                        mesh_instance.transform.matrix(),
                        color,
                    );
                }
            }
        }
    }

    let camera = frame.effective_camera();
    for anchor in &frame.overlays().selection_anchors {
        append_cross(
            &mut vertices,
            anchor.position,
            anchor.size,
            anchor.color,
            camera.transform.right(),
            camera.transform.up(),
        );
    }

    vertices
}

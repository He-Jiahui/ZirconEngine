use crate::core::framework::render::DisplayMode;
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
    for highlight in &frame.scene.overlays.selection {
        let Some(mesh_instance) = frame
            .scene
            .scene
            .meshes
            .iter()
            .find(|mesh| mesh.node_id == highlight.owner)
        else {
            continue;
        };
        let Some(model) = streamer.model(&mesh_instance.model.id()) else {
            continue;
        };
        let color = if frame.scene.overlays.display_mode == DisplayMode::WireOnly {
            Vec4::new(1.0, 0.88, 0.32, 1.0)
        } else {
            Vec4::new(1.0, 0.76, 0.18, 1.0)
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

    for anchor in &frame.scene.overlays.selection_anchors {
        append_cross(
            &mut vertices,
            anchor.position,
            anchor.size,
            anchor.color,
            frame.scene.scene.camera.transform.right(),
            frame.scene.scene.camera.transform.up(),
        );
    }

    vertices
}

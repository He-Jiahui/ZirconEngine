use std::collections::HashSet;

use crate::core::framework::render::DisplayMode;
use crate::core::math::Vec4;

use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::primitives::LineVertex;
use crate::graphics::types::EditorOrRuntimeFrame;

pub(crate) fn build_wireframe_vertices(
    frame: &EditorOrRuntimeFrame,
    streamer: &ResourceStreamer,
) -> Vec<LineVertex> {
    let selection: HashSet<_> = frame
        .scene
        .overlays
        .selection
        .iter()
        .map(|highlight| highlight.owner)
        .collect();

    let mut vertices = Vec::new();
    for mesh_instance in &frame.scene.scene.meshes {
        let Some(model) = streamer.model(&mesh_instance.model.id()) else {
            continue;
        };
        let color = match frame.scene.overlays.display_mode {
            DisplayMode::WireOverlay => Vec4::new(0.08, 0.09, 0.1, 0.9),
            DisplayMode::WireOnly => {
                if selection.contains(&mesh_instance.node_id) {
                    Vec4::new(1.0, 0.9, 0.45, 1.0)
                } else {
                    Vec4::new(0.86, 0.88, 0.93, 1.0)
                }
            }
            DisplayMode::Shaded => Vec4::ONE,
        };
        let model_matrix = mesh_instance.transform.matrix();
        for mesh in &model.meshes {
            for [start, end] in &mesh.wire_segments {
                vertices.push(LineVertex::new(
                    model_matrix.transform_point3(*start),
                    color,
                ));
                vertices.push(LineVertex::new(model_matrix.transform_point3(*end), color));
            }
        }
    }
    vertices
}

use std::collections::HashSet;

use crate::core::framework::render::DisplayMode;
use crate::core::math::Vec4;

use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::primitives::LineVertex;
use crate::graphics::types::ViewportRenderFrame;

pub(crate) fn build_wireframe_vertices(
    frame: &ViewportRenderFrame,
    streamer: &ResourceStreamer,
) -> Vec<LineVertex> {
    let display_mode = frame.overlays().display_mode;
    if display_mode == DisplayMode::Shaded {
        return Vec::new();
    }
    let selection: HashSet<_> = if display_mode == DisplayMode::WireOnly {
        frame
            .overlays()
            .selection
            .iter()
            .map(|highlight| highlight.owner)
            .collect()
    } else {
        HashSet::new()
    };

    let mut vertices = Vec::new();
    for mesh_instance in frame.meshes() {
        let Some(model) = streamer.model(&mesh_instance.model.id()) else {
            continue;
        };
        let color = match display_mode {
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

#[cfg(test)]
mod tests {
    #[test]
    fn shaded_wireframe_skips_vertex_and_selection_build_before_iteration() {
        let source = include_str!("build_wireframe_vertices.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("wireframe primitive implementation");
        let shaded_guard = implementation
            .find("if display_mode == DisplayMode::Shaded")
            .expect("Shaded early return");
        let selection_build = implementation
            .find("let selection: HashSet")
            .expect("WireOnly selection index");
        let mesh_loop = implementation
            .find("for mesh_instance in frame.meshes()")
            .expect("wireframe mesh loop");

        assert!(shaded_guard < selection_build);
        assert!(shaded_guard < mesh_loop);
        assert!(implementation.contains("display_mode == DisplayMode::WireOnly"));
    }
}

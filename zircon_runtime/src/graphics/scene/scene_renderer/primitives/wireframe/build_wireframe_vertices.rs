use std::collections::HashSet;

use crate::core::framework::render::DisplayMode;
use crate::core::math::Vec4;

use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::primitives::LineVertex;
use crate::graphics::types::ViewportRenderFrame;

/// 将资源中的 wire_segments 投到实例世界空间；Shaded 直接跳过，WireOnly 对选中实体使用强调色。
pub(crate) fn build_wireframe_vertices(
    frame: &ViewportRenderFrame,
    streamer: &ResourceStreamer,
) -> Vec<LineVertex> {
    let display_mode = frame.overlays().display_mode;
    if display_mode == DisplayMode::Shaded {
        return Vec::new();
    }
    let selection = if display_mode == DisplayMode::WireOnly {
        if let Some(highlights) = frame.overlays().highlights.as_ref() {
            let entities = highlights.entities();
            let mut selection = HashSet::with_capacity(entities.len());
            selection.extend(entities.iter().copied());
            selection
        } else {
            HashSet::new()
        }
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
            vertices.reserve(mesh.wire_segments.len().saturating_mul(2));
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
#[path = "tests/build_wireframe_vertices.rs"]
mod tests;

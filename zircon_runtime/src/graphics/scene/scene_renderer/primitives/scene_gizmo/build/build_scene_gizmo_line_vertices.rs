use crate::core::framework::render::{SceneGizmoOverlayExtract, ViewportIconId};

use crate::graphics::scene::scene_renderer::primitives::LineVertex;
use crate::graphics::types::ViewportRenderFrame;

use super::super::super::line_geometry::push_line;
use super::super::append::{
    append_icon_fallback_lines, append_wire_shape, icon_fallback_vertex_capacity,
    wire_shape_vertex_capacity,
};

const LINE_VERTEX_CAPACITY: usize = 2;

fn scene_gizmo_line_vertex_capacity<F>(
    gizmos: &[SceneGizmoOverlayExtract],
    has_icon_texture: &F,
) -> usize
where
    F: Fn(ViewportIconId) -> bool,
{
    gizmos
        .iter()
        .map(|gizmo| {
            let line_capacity = gizmo.lines.len().saturating_mul(LINE_VERTEX_CAPACITY);
            let wire_capacity = gizmo
                .wire_shapes
                .iter()
                .map(wire_shape_vertex_capacity)
                .fold(0usize, usize::saturating_add);
            let icon_capacity = gizmo
                .icons
                .iter()
                .filter(|icon| !has_icon_texture(icon.id))
                .map(icon_fallback_vertex_capacity)
                .fold(0usize, usize::saturating_add);
            line_capacity
                .saturating_add(wire_capacity)
                .saturating_add(icon_capacity)
        })
        .fold(0usize, usize::saturating_add)
}

/// 聚合已抽取的 gizmo 线、线框形状及缺失图标回退；贴图图标由 scene_gizmo_pass 另行准备。
/// has_icon_texture 必须与本帧 atlas 准备状态一致，以免贴图与线框同时出现或同时缺席。
pub(crate) fn build_scene_gizmo_line_vertices<F>(
    frame: &ViewportRenderFrame,
    has_icon_texture: F,
) -> Vec<LineVertex>
where
    F: Fn(ViewportIconId) -> bool,
{
    let vertex_capacity =
        scene_gizmo_line_vertex_capacity(&frame.overlays().scene_gizmos, &has_icon_texture);
    let mut vertices = Vec::with_capacity(vertex_capacity);
    let camera = frame.effective_camera();
    let camera_right = camera.transform.right();
    let camera_up = camera.transform.up();
    for gizmo in &frame.overlays().scene_gizmos {
        for line in &gizmo.lines {
            push_line(&mut vertices, line);
        }
        for shape in &gizmo.wire_shapes {
            append_wire_shape(&mut vertices, shape);
        }
        for icon in &gizmo.icons {
            if !has_icon_texture(icon.id) {
                append_icon_fallback_lines(&mut vertices, icon, camera_right, camera_up);
            }
        }
    }
    vertices
}

#[cfg(test)]
#[path = "tests/build_scene_gizmo_line_vertices.rs"]
mod tests;

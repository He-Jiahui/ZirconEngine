use crate::core::framework::render::ViewportIconId;

use crate::graphics::scene::scene_renderer::primitives::LineVertex;
use crate::graphics::types::EditorOrRuntimeFrame;

use super::super::super::line_geometry::push_line;
use super::super::append::{append_icon_fallback_lines, append_wire_shape};

pub(crate) fn build_scene_gizmo_line_vertices<F>(
    frame: &EditorOrRuntimeFrame,
    has_icon_texture: F,
) -> Vec<LineVertex>
where
    F: Fn(ViewportIconId) -> bool,
{
    let mut vertices = Vec::new();
    let camera = &frame.scene.scene.camera;
    let camera_right = camera.transform.right();
    let camera_up = camera.transform.up();
    for gizmo in &frame.scene.overlays.scene_gizmos {
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

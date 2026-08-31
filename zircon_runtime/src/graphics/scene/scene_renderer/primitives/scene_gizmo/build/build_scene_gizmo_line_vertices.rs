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
mod tests {
    use crate::core::framework::render::{
        OverlayBillboardIcon, OverlayLineSegment, OverlayWireShape, SceneGizmoKind,
        SceneGizmoOverlayExtract, ViewportIconId,
    };
    use crate::core::math::{Vec3, Vec4};

    use super::scene_gizmo_line_vertex_capacity;

    #[test]
    fn scene_gizmo_line_capacity_counts_only_missing_icon_fallbacks() {
        let gizmos = [SceneGizmoOverlayExtract::new(
            1,
            SceneGizmoKind::Camera,
            false,
            vec![OverlayLineSegment {
                start: Vec3::ZERO,
                end: Vec3::X,
                color: Vec4::ONE,
            }],
            vec![OverlayWireShape::Arrow {
                origin: Vec3::ZERO,
                direction: Vec3::X,
                length: 1.0,
                color: Vec4::ONE,
            }],
            vec![
                OverlayBillboardIcon {
                    id: ViewportIconId::Camera,
                    position: Vec3::ZERO,
                    tint: Vec4::ONE,
                    size: 1.0,
                },
                OverlayBillboardIcon {
                    id: ViewportIconId::DirectionalLight,
                    position: Vec3::ONE,
                    tint: Vec4::ONE,
                    size: 1.0,
                },
            ],
            Vec::new(),
        )];

        assert_eq!(scene_gizmo_line_vertex_capacity(&gizmos, &|_| false), 28);
        assert_eq!(
            scene_gizmo_line_vertex_capacity(&gizmos, &|id| id == ViewportIconId::Camera),
            16
        );
        assert_eq!(scene_gizmo_line_vertex_capacity(&gizmos, &|_| true), 8);
    }
}

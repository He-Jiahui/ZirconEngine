use crate::scene::viewport::pointer::{
    ViewportOverlayPointerBridge, ViewportPointerLayout, ViewportPointerRoute,
    ViewportRenderablePickCandidate,
};
use crate::GizmoAxis;
use crate::scene::viewport::{
    HandleElementExtract, HandleOverlayExtract, OverlayAxis, OverlayPickShape, ProjectionMode,
    SceneGizmoKind, SceneGizmoOverlayExtract, ViewportCameraSnapshot,
};
use zircon_runtime::core::math::{perspective, view_matrix, Transform, UVec2, Vec2, Vec3, Vec4};
use zircon_runtime::ui::layout::UiPoint;

#[test]
fn viewport_overlay_pointer_bridge_prefers_handle_axis_over_renderable_candidate() {
    let viewport = UVec2::new(1280, 720);
    let camera = test_camera();
    let start = Vec3::new(0.0, 0.0, 0.0);
    let end = Vec3::new(2.0, 0.0, 0.0);
    let cursor = projected_point(&camera, viewport, Vec3::new(1.0, 0.0, 0.0));

    let mut bridge = ViewportOverlayPointerBridge::new();
    bridge.sync(ViewportPointerLayout {
        viewport,
        camera: camera.clone(),
        handles: vec![HandleOverlayExtract {
            owner: 7,
            origin: Transform::default(),
            elements: vec![HandleElementExtract::AxisLine {
                axis: OverlayAxis::X,
                start,
                end,
                color: Vec4::ONE,
                pick_radius: 0.15,
            }],
        }],
        scene_gizmos: Vec::new(),
        renderables: vec![ViewportRenderablePickCandidate {
            owner: 99,
            position: Vec3::new(1.0, 0.0, 0.0),
            radius_world: 1.2,
        }],
    });

    let dispatch = bridge
        .handle_move(UiPoint::new(cursor.x, cursor.y))
        .expect("shared viewport route should resolve handle hover");
    assert_eq!(
        dispatch.route,
        Some(ViewportPointerRoute::HandleAxis {
            owner: 7,
            axis: GizmoAxis::X,
        })
    );
}

#[test]
fn viewport_overlay_pointer_bridge_prefers_scene_gizmo_over_renderable_candidate() {
    let viewport = UVec2::new(1280, 720);
    let camera = test_camera();
    let gizmo_center = Vec3::new(0.5, 0.0, 0.0);
    let cursor = projected_point(&camera, viewport, gizmo_center);

    let mut bridge = ViewportOverlayPointerBridge::new();
    bridge.sync(ViewportPointerLayout {
        viewport,
        camera: camera.clone(),
        handles: Vec::new(),
        scene_gizmos: vec![SceneGizmoOverlayExtract {
            owner: 41,
            kind: SceneGizmoKind::DirectionalLight,
            selected: false,
            lines: Vec::new(),
            wire_shapes: Vec::new(),
            icons: Vec::new(),
            pick_shapes: vec![OverlayPickShape::Sphere {
                center: gizmo_center,
                radius: 0.4,
            }],
        }],
        renderables: vec![ViewportRenderablePickCandidate {
            owner: 88,
            position: gizmo_center,
            radius_world: 1.2,
        }],
    });

    let dispatch = bridge
        .handle_move(UiPoint::new(cursor.x, cursor.y))
        .expect("shared viewport route should resolve gizmo hover");
    assert_eq!(
        dispatch.route,
        Some(ViewportPointerRoute::SceneGizmo { owner: 41 })
    );
}

#[test]
fn viewport_overlay_pointer_bridge_resolves_renderable_when_no_overlay_hits() {
    let viewport = UVec2::new(1280, 720);
    let camera = test_camera();
    let cursor = projected_point(&camera, viewport, Vec3::new(-0.75, 0.1, 0.0));

    let mut bridge = ViewportOverlayPointerBridge::new();
    bridge.sync(ViewportPointerLayout {
        viewport,
        camera: camera.clone(),
        handles: Vec::new(),
        scene_gizmos: Vec::new(),
        renderables: vec![ViewportRenderablePickCandidate {
            owner: 13,
            position: Vec3::new(-0.75, 0.1, 0.0),
            radius_world: 1.0,
        }],
    });

    let dispatch = bridge
        .handle_move(UiPoint::new(cursor.x, cursor.y))
        .expect("shared viewport route should resolve renderable hover");
    assert_eq!(
        dispatch.route,
        Some(ViewportPointerRoute::Renderable { owner: 13 })
    );
}

fn test_camera() -> ViewportCameraSnapshot {
    ViewportCameraSnapshot {
        transform: Transform::looking_at(Vec3::new(0.0, 0.0, 8.0), Vec3::ZERO, Vec3::Y),
        projection_mode: ProjectionMode::Perspective,
        fov_y_radians: 60.0_f32.to_radians(),
        ortho_size: 5.0,
        z_near: 0.1,
        z_far: 200.0,
        aspect_ratio: 1280.0 / 720.0,
    }
}

fn projected_point(camera: &ViewportCameraSnapshot, viewport: UVec2, world: Vec3) -> Vec2 {
    let aspect = viewport.x as f32 / viewport.y.max(1) as f32;
    let projection = match camera.projection_mode {
        ProjectionMode::Perspective => {
            perspective(camera.fov_y_radians, aspect, camera.z_near, camera.z_far)
        }
        ProjectionMode::Orthographic => {
            let half_height = camera.ortho_size.max(0.01);
            let half_width = half_height * aspect.max(0.001);
            zircon_runtime::core::math::Mat4::orthographic_rh(
                -half_width,
                half_width,
                -half_height,
                half_height,
                camera.z_near.max(0.001),
                camera.z_far,
            )
        }
    };
    let clip = projection * view_matrix(camera.transform) * world.extend(1.0);
    let ndc = clip.truncate() / clip.w;
    Vec2::new(
        (ndc.x * 0.5 + 0.5) * viewport.x as f32,
        (-ndc.y * 0.5 + 0.5) * viewport.y as f32,
    )
}

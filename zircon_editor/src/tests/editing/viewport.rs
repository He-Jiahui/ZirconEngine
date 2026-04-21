use crate::scene::viewport::pointer::{
    ViewportOverlayPointerRouter, ViewportPointerLayout, ViewportPointerRoute,
    ViewportRenderablePickCandidate,
};
use crate::scene::viewport::{
    GizmoAxis, GridMode, HandleElementExtract, HandleOverlayExtract, OverlayAxis, OverlayPickShape,
    ProjectionMode, SceneGizmoKind, SceneGizmoOverlayExtract, SceneViewportController,
    ViewportCameraSnapshot,
};
use zircon_runtime::core::math::{perspective, view_matrix, Transform, UVec2, Vec2, Vec3, Vec4};
use zircon_runtime::scene::Scene;
use zircon_runtime::ui::layout::UiPoint;

#[test]
fn viewport_overlay_pointer_router_prefers_handle_axis_over_renderable_candidate() {
    let viewport = UVec2::new(1280, 720);
    let camera = test_camera();
    let start = Vec3::new(0.0, 0.0, 0.0);
    let end = Vec3::new(2.0, 0.0, 0.0);
    let cursor = projected_point(&camera, viewport, Vec3::new(1.0, 0.0, 0.0));

    let mut router = ViewportOverlayPointerRouter::new();
    router.sync(ViewportPointerLayout {
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

    let dispatch = router
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
fn viewport_overlay_pointer_router_prefers_scene_gizmo_over_renderable_candidate() {
    let viewport = UVec2::new(1280, 720);
    let camera = test_camera();
    let gizmo_center = Vec3::new(0.5, 0.0, 0.0);
    let cursor = projected_point(&camera, viewport, gizmo_center);

    let mut router = ViewportOverlayPointerRouter::new();
    router.sync(ViewportPointerLayout {
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

    let dispatch = router
        .handle_move(UiPoint::new(cursor.x, cursor.y))
        .expect("shared viewport route should resolve gizmo hover");
    assert_eq!(
        dispatch.route,
        Some(ViewportPointerRoute::SceneGizmo { owner: 41 })
    );
}

#[test]
fn viewport_overlay_pointer_router_resolves_renderable_when_no_overlay_hits() {
    let viewport = UVec2::new(1280, 720);
    let camera = test_camera();
    let cursor = projected_point(&camera, viewport, Vec3::new(-0.75, 0.1, 0.0));

    let mut router = ViewportOverlayPointerRouter::new();
    router.sync(ViewportPointerLayout {
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

    let dispatch = router
        .handle_move(UiPoint::new(cursor.x, cursor.y))
        .expect("shared viewport route should resolve renderable hover");
    assert_eq!(
        dispatch.route,
        Some(ViewportPointerRoute::Renderable { owner: 13 })
    );
}

#[test]
fn viewport_render_snapshot_keeps_authoring_overlay_and_preview_state_in_editor_only() {
    let scene = Scene::new();
    let selected = scene
        .nodes()
        .iter()
        .find(|node| node.directional_light.is_some())
        .unwrap()
        .id;
    let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
    controller.set_selected_node(Some(selected));

    let authored = controller.build_render_snapshot(&scene);
    assert_eq!(authored.overlays.selection.len(), 1);
    assert!(authored.overlays.selection_anchors.is_empty());
    assert!(authored.overlays.grid.is_some());
    assert!(!authored.overlays.scene_gizmos.is_empty());
    assert!(authored.preview.lighting_enabled);
    assert!(authored.preview.skybox_enabled);

    controller.settings_mut().gizmos_enabled = false;
    controller.settings_mut().grid_mode = GridMode::Hidden;
    controller.settings_mut().preview_lighting = false;
    controller.settings_mut().preview_skybox = false;

    let toggled = controller.build_render_snapshot(&scene);
    assert_eq!(toggled.overlays.selection.len(), 1);
    assert_eq!(toggled.overlays.selection_anchors.len(), 1);
    assert!(toggled.overlays.grid.is_none());
    assert!(toggled.overlays.scene_gizmos.is_empty());
    assert!(!toggled.preview.lighting_enabled);
    assert!(!toggled.preview.skybox_enabled);

    let runtime_packet = scene.to_render_extract();
    assert!(runtime_packet.overlays.selection.is_empty());
    assert!(runtime_packet.overlays.selection_anchors.is_empty());
    assert!(runtime_packet.overlays.handles.is_empty());
    assert!(runtime_packet.overlays.scene_gizmos.is_empty());
    assert!(runtime_packet.overlays.grid.is_none());
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

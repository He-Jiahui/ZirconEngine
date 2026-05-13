use crate::scene::viewport::pointer::{
    ViewportOverlayPointerRouter, ViewportPointerLayout, ViewportPointerRoute,
    ViewportRenderablePickCandidate,
};
use crate::scene::viewport::{
    GizmoAxis, GridMode, HandleElementExtract, HandleOverlayExtract, OverlayAxis, OverlayPickShape,
    ProjectionMode, SceneGizmoKind, SceneGizmoOverlayExtract, SceneInspectorFieldValue,
    SceneViewportController, SceneViewportTool, TransformSpace, ViewportCameraSnapshot,
};
use zircon_runtime::core::framework::picking::{HitTarget, PickingAxis};
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::{
    perspective, view_matrix, Transform, UVec2, Vec2, Vec3, Vec4,
};
use zircon_runtime_interface::ui::layout::UiPoint;

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
fn viewport_overlay_pointer_router_skips_rebuild_for_unchanged_layout() {
    let viewport = UVec2::new(1280, 720);
    let camera = test_camera();
    let layout = ViewportPointerLayout {
        viewport,
        camera: camera.clone(),
        handles: Vec::new(),
        scene_gizmos: Vec::new(),
        renderables: vec![ViewportRenderablePickCandidate {
            owner: 13,
            position: Vec3::new(-0.75, 0.1, 0.0),
            radius_world: 1.0,
        }],
    };

    let mut router = ViewportOverlayPointerRouter::new();

    assert!(router.sync(layout.clone()));
    assert!(!router.sync(layout));
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
fn viewport_pointer_route_maps_to_runtime_hit_target_contract() {
    let route = ViewportPointerRoute::HandleAxis {
        owner: 7,
        axis: GizmoAxis::Y,
    };

    assert_eq!(
        route.target(),
        HitTarget::handle_axis(7, PickingAxis::Y),
        "editor routes should adapt to the runtime picking target contract"
    );
    assert_eq!(
        ViewportPointerRoute::from_target(HitTarget::scene_gizmo(41)),
        ViewportPointerRoute::SceneGizmo { owner: 41 }
    );
}

#[test]
fn viewport_pointer_route_priority_matches_runtime_hit_target_order() {
    assert!(
        HitTarget::handle_axis(1, PickingAxis::X).priority() < HitTarget::scene_gizmo(2).priority()
    );
    assert!(HitTarget::scene_gizmo(2).priority() < HitTarget::renderable(3).priority());
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

#[test]
fn viewport_edit_mode_projection_derives_authoring_panels_from_runtime_world() {
    let mut scene = Scene::new();
    let cube = scene
        .nodes()
        .iter()
        .find(|node| node.kind == NodeKind::Cube)
        .expect("default cube")
        .id;
    let child = scene.spawn_node(NodeKind::PointLight);
    scene.rename_node(cube, "Root Cube").unwrap();
    scene.set_parent_checked(child, Some(cube)).unwrap();
    scene.set_active_self(child, false).unwrap();

    let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
    controller.set_selected_node(Some(cube));
    controller.settings_mut().tool = SceneViewportTool::Scale;
    controller.settings_mut().transform_space = TransformSpace::Global;
    controller.settings_mut().projection_mode = ProjectionMode::Orthographic;
    controller.settings_mut().grid_mode = GridMode::VisibleAndSnap;

    let projection = controller.build_edit_mode_projection(&scene);

    assert_eq!(projection.selected_entity, Some(cube));
    let root_row = projection
        .hierarchy_rows
        .iter()
        .find(|row| row.entity == cube)
        .expect("selected cube row");
    assert_eq!(root_row.parent, None);
    assert_eq!(root_row.depth, 0);
    assert_eq!(root_row.display_name, "Root Cube");
    assert!(root_row.selected);
    assert!(root_row.active_in_hierarchy);
    assert!(root_row.has_children);

    let child_row = projection
        .hierarchy_rows
        .iter()
        .find(|row| row.entity == child)
        .expect("child light row");
    assert_eq!(child_row.parent, Some(cube));
    assert_eq!(child_row.depth, 1);
    assert!(!child_row.selected);
    assert!(!child_row.active_in_hierarchy);

    assert_eq!(projection.toolbar.tool, SceneViewportTool::Scale);
    assert_eq!(projection.toolbar.transform_space, TransformSpace::Global);
    assert_eq!(
        projection.toolbar.projection_mode,
        ProjectionMode::Orthographic
    );
    assert_eq!(projection.toolbar.grid_mode, GridMode::VisibleAndSnap);
    assert!(projection.toolbar.has_selection);
    assert!(projection.toolbar.can_frame_selection);

    assert_eq!(projection.stats.node_count, scene.node_records().len());
    assert_eq!(projection.stats.camera_count, 1);
    assert_eq!(projection.stats.mesh_count, 1);
    assert_eq!(projection.stats.light_count, 2);
    assert_eq!(projection.stats.visible_node_count, 3);

    assert!(projection.inspector_fields.iter().any(|field| {
        field.property_path.as_deref() == Some("Name.value")
            && field.value == SceneInspectorFieldValue::Text("Root Cube".to_string())
            && field.editable
    }));
    assert!(projection.inspector_fields.iter().any(|field| {
        field.property_path.as_deref() == Some("Transform.translation")
            && matches!(field.value, SceneInspectorFieldValue::Vec3(_))
            && field.editable
    }));
    assert!(projection.inspector_fields.iter().any(|field| {
        field.property_path.as_deref() == Some("MeshRenderer.model")
            && matches!(field.value, SceneInspectorFieldValue::Resource(_))
            && !field.editable
    }));

    let runtime_packet = scene.to_render_extract();
    assert!(runtime_packet.overlays.selection.is_empty());
    assert!(runtime_packet.overlays.scene_gizmos.is_empty());
}

#[test]
fn viewport_edit_mode_projection_ignores_stale_editor_selection() {
    let scene = Scene::new();
    let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
    controller.set_selected_node(Some(999_999));

    let projection = controller.build_edit_mode_projection(&scene);

    assert_eq!(projection.selected_entity, None);
    assert!(projection.inspector_fields.is_empty());
    assert!(!projection.toolbar.has_selection);
    assert!(!projection.toolbar.can_frame_selection);
    assert!(projection.hierarchy_rows.iter().all(|row| !row.selected));
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
            zircon_runtime_interface::math::Mat4::orthographic_rh(
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

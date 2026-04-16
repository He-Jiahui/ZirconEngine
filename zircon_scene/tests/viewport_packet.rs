use zircon_math::UVec2;
use zircon_scene::{
    DisplayMode, GridMode, NodeKind, ProjectionMode, SceneGizmoKind, SceneViewportExtractRequest,
    SceneViewportSettings, SceneViewportTool, TransformSpace, ViewOrientation, World,
};

fn default_settings() -> SceneViewportSettings {
    SceneViewportSettings {
        tool: SceneViewportTool::Move,
        transform_space: TransformSpace::Local,
        projection_mode: ProjectionMode::Perspective,
        view_orientation: ViewOrientation::User,
        gizmos_enabled: true,
        display_mode: DisplayMode::Shaded,
        grid_mode: GridMode::VisibleAndSnap,
        translate_step: 1.0,
        rotate_step_deg: 15.0,
        scale_step: 0.1,
        preview_lighting: true,
        preview_skybox: true,
    }
}

#[test]
fn selected_renderable_produces_selection_highlight_without_scene_gizmo_icon() {
    let world = World::new();
    let cube = world
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::Cube))
        .map(|node| node.id)
        .expect("default scene cube");

    let request = SceneViewportExtractRequest {
        settings: default_settings(),
        selection: vec![cube],
        active_camera_override: None,
        camera: None,
        viewport_size: Some(UVec2::new(1280, 720)),
    };

    let packet = world.build_viewport_render_packet(&request);

    assert!(packet
        .overlays
        .selection
        .iter()
        .any(|highlight| highlight.owner == cube && highlight.outline));
    assert!(packet
        .overlays
        .scene_gizmos
        .iter()
        .all(|gizmo| gizmo.owner != cube));
}

#[test]
fn gizmo_registry_outputs_camera_and_directional_light_when_enabled() {
    let world = World::new();
    let camera = world
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::Camera))
        .map(|node| node.id)
        .expect("default scene camera");
    let light = world
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::DirectionalLight))
        .map(|node| node.id)
        .expect("default scene directional light");

    let packet = world.build_viewport_render_packet(&SceneViewportExtractRequest {
        settings: default_settings(),
        selection: vec![camera, light],
        active_camera_override: None,
        camera: None,
        viewport_size: Some(UVec2::new(1280, 720)),
    });

    assert!(packet.overlays.scene_gizmos.iter().any(|gizmo| {
        gizmo.owner == camera && gizmo.kind == SceneGizmoKind::Camera && gizmo.selected
    }));
    assert!(packet.overlays.scene_gizmos.iter().any(|gizmo| {
        gizmo.owner == light && gizmo.kind == SceneGizmoKind::DirectionalLight && gizmo.selected
    }));
}

#[test]
fn viewport_settings_drive_grid_preview_and_gizmo_visibility() {
    let world = World::new();
    let camera = world
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::Camera))
        .map(|node| node.id)
        .expect("default scene camera");
    let mut settings = default_settings();
    settings.gizmos_enabled = false;
    settings.display_mode = DisplayMode::WireOnly;
    settings.grid_mode = GridMode::VisibleNoSnap;
    settings.preview_lighting = false;
    settings.preview_skybox = false;

    let packet = world.build_viewport_render_packet(&SceneViewportExtractRequest {
        settings,
        selection: vec![camera],
        active_camera_override: None,
        camera: None,
        viewport_size: Some(UVec2::new(1280, 720)),
    });

    assert_eq!(packet.overlays.display_mode, DisplayMode::WireOnly);
    assert!(packet.overlays.grid.is_some());
    assert!(
        !packet
            .overlays
            .grid
            .as_ref()
            .expect("grid extract")
            .snap_enabled
    );
    assert!(packet.overlays.scene_gizmos.is_empty());
    assert!(!packet.preview.lighting_enabled);
    assert!(!packet.preview.skybox_enabled);
}

#[test]
fn viewport_request_propagates_viewport_aspect_ratio_into_camera_and_gizmo_extracts() {
    let world = World::new();
    let camera = world
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::Camera))
        .map(|node| node.id)
        .expect("default scene camera");

    let packet = world.build_viewport_render_packet(&SceneViewportExtractRequest {
        settings: default_settings(),
        selection: vec![camera],
        active_camera_override: None,
        camera: None,
        viewport_size: Some(UVec2::new(1024, 512)),
    });

    assert!((packet.scene.camera.aspect_ratio - 2.0).abs() < 0.0001);
    assert!(packet.overlays.scene_gizmos.iter().any(|gizmo| {
        gizmo.owner == camera
            && gizmo.kind == SceneGizmoKind::Camera
            && gizmo.wire_shapes.iter().any(|shape| {
                matches!(
                    shape,
                    zircon_scene::OverlayWireShape::Frustum { aspect_ratio, .. }
                        if (*aspect_ratio - 2.0).abs() < 0.0001
                )
            })
    }));
}

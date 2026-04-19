use zircon_framework::render::{
    DisplayMode, ProjectionMode, SceneViewportExtractRequest, ViewportRenderSettings,
};
use zircon_math::UVec2;
use zircon_scene::world::World;

fn default_settings() -> ViewportRenderSettings {
    ViewportRenderSettings {
        projection_mode: ProjectionMode::Perspective,
        display_mode: DisplayMode::Shaded,
        preview_lighting: true,
        preview_skybox: true,
    }
}

#[test]
fn runtime_world_does_not_emit_editor_owned_overlays() {
    let world = World::new();

    let request = SceneViewportExtractRequest {
        settings: default_settings(),
        active_camera_override: None,
        camera: None,
        viewport_size: Some(UVec2::new(1280, 720)),
    };

    let packet = world.build_viewport_render_packet(&request);

    assert!(packet.overlays.selection.is_empty());
    assert!(packet.overlays.selection_anchors.is_empty());
    assert!(packet.overlays.handles.is_empty());
    assert!(packet.overlays.scene_gizmos.is_empty());
    assert!(packet.overlays.grid.is_none());
}

#[test]
fn runtime_world_does_not_emit_scene_gizmos() {
    let world = World::new();

    let packet = world.build_viewport_render_packet(&SceneViewportExtractRequest {
        settings: default_settings(),
        active_camera_override: None,
        camera: None,
        viewport_size: Some(UVec2::new(1280, 720)),
    });

    assert!(packet.overlays.scene_gizmos.is_empty());
    assert!(packet.overlays.selection.is_empty());
    assert!(packet.overlays.selection_anchors.is_empty());
    assert!(packet.overlays.handles.is_empty());
}

#[test]
fn viewport_settings_drive_preview_flags_but_not_editor_overlays() {
    let world = World::new();
    let mut settings = default_settings();
    settings.display_mode = DisplayMode::WireOnly;
    settings.preview_lighting = false;
    settings.preview_skybox = false;

    let packet = world.build_viewport_render_packet(&SceneViewportExtractRequest {
        settings,
        active_camera_override: None,
        camera: None,
        viewport_size: Some(UVec2::new(1280, 720)),
    });

    assert_eq!(packet.overlays.display_mode, DisplayMode::WireOnly);
    assert!(packet.overlays.grid.is_none());
    assert!(packet.overlays.scene_gizmos.is_empty());
    assert!(packet.overlays.selection.is_empty());
    assert!(packet.overlays.selection_anchors.is_empty());
    assert!(packet.overlays.handles.is_empty());
    assert!(!packet.preview.lighting_enabled);
    assert!(!packet.preview.skybox_enabled);
}

#[test]
fn viewport_request_propagates_viewport_aspect_ratio_into_runtime_camera_extract() {
    let world = World::new();
    let packet = world.build_viewport_render_packet(&SceneViewportExtractRequest {
        settings: default_settings(),
        active_camera_override: None,
        camera: None,
        viewport_size: Some(UVec2::new(1024, 512)),
    });

    assert!((packet.scene.camera.aspect_ratio - 2.0).abs() < 0.0001);
    assert!(packet.overlays.scene_gizmos.is_empty());
    assert!(packet.overlays.selection.is_empty());
    assert!(packet.overlays.selection_anchors.is_empty());
    assert!(packet.overlays.handles.is_empty());
}

use zircon_scene::{
    Scene, SceneGizmoOverlayExtract, SceneViewportExtractRequest, SceneViewportSettings,
    ViewportCameraSnapshot,
};

pub(in crate::editing::viewport::pointer) fn scene_gizmo_candidates(
    scene: &Scene,
    settings: &SceneViewportSettings,
    camera: &ViewportCameraSnapshot,
) -> Vec<SceneGizmoOverlayExtract> {
    if !settings.gizmos_enabled {
        return Vec::new();
    }
    scene
        .build_viewport_render_packet(&SceneViewportExtractRequest {
            settings: settings.clone(),
            selection: scene.selected_node().into_iter().collect(),
            active_camera_override: None,
            camera: Some(camera.clone()),
            viewport_size: None,
        })
        .overlays
        .scene_gizmos
}

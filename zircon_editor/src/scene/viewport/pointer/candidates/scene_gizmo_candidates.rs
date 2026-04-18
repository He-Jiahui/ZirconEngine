use zircon_scene::{
    Scene, SceneGizmoOverlayExtract, SceneViewportSettings, ViewportCameraSnapshot,
};

use crate::scene::viewport::render_packet::build_scene_gizmos;

pub(in crate::scene::viewport::pointer) fn scene_gizmo_candidates(
    scene: &Scene,
    selected: Option<u64>,
    settings: &SceneViewportSettings,
    camera: &ViewportCameraSnapshot,
) -> Vec<SceneGizmoOverlayExtract> {
    build_scene_gizmos(scene, selected, settings, camera)
}

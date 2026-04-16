use zircon_scene::{RenderSceneSnapshot, Scene, SceneViewportExtractRequest};

use super::SceneViewportController;

impl SceneViewportController {
    pub(crate) fn build_render_snapshot(&self, scene: &Scene) -> RenderSceneSnapshot {
        let camera = self.current_camera(scene);
        let handles = self.handle_overlays(scene, &camera);
        let mut packet = scene.build_viewport_render_packet(&SceneViewportExtractRequest {
            settings: self.state.settings.clone(),
            selection: scene.selected_node().into_iter().collect(),
            active_camera_override: None,
            camera: Some(camera),
            viewport_size: Some(self.state.viewport.size),
        });
        packet.overlays.handles = handles;
        packet
    }
}

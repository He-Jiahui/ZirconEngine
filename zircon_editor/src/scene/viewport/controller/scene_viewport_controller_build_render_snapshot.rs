use zircon_scene::{RenderSceneSnapshot, Scene};

use super::SceneViewportController;
use crate::scene::viewport::render_packet::build_render_packet;

impl SceneViewportController {
    pub(crate) fn build_render_snapshot(&self, scene: &Scene) -> RenderSceneSnapshot {
        let camera = self.current_camera(scene);
        let handles = self.handle_overlays(scene, &camera);
        build_render_packet(
            scene,
            &self.state.settings,
            &camera,
            self.selected_node(),
            self.state.viewport.size,
            handles,
        )
    }
}

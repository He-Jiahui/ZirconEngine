use crate::scene::viewport::RenderSceneSnapshot;
use zircon_runtime::scene::Scene;

use super::SceneViewportController;
use crate::scene::viewport::render_packet::build_render_packet;

impl SceneViewportController {
    pub(crate) fn build_render_snapshot(&self, scene: &Scene) -> RenderSceneSnapshot {
        let camera = self.current_camera(scene);
        let selected = self.projected_selected_node(scene);
        let handles = self.handle_overlays(scene, &camera);
        build_render_packet(
            scene,
            &self.state.settings,
            &camera,
            selected,
            self.state.viewport.size,
            handles,
        )
    }
}

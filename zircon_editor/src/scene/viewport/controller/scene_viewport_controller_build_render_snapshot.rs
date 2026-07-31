use crate::scene::viewport::RenderSceneSnapshot;
use zircon_runtime::scene::Scene;

use super::SceneViewportController;
use crate::scene::viewport::render_packet::{apply_interaction_overlays, build_render_packet};

impl SceneViewportController {
    pub(crate) fn build_render_snapshot(&self, scene: &Scene) -> RenderSceneSnapshot {
        let camera = self.current_camera(scene);
        let selected = self.projected_selected_node(scene);
        let mut packet = build_render_packet(
            scene,
            &self.state.settings,
            &camera,
            selected,
            self.state.viewport.size,
        );
        let interaction_extract = self.interaction_extract.resolve_from_render_packet(
            scene,
            selected,
            &self.state.settings,
            &camera,
            self.state.viewport.size,
            &packet.scene.meshes,
            || self.handle_overlays(scene, &camera),
            || self.viewport_overlay_gizmos(scene, selected),
        );
        apply_interaction_overlays(&mut packet, &interaction_extract);
        packet
    }
}

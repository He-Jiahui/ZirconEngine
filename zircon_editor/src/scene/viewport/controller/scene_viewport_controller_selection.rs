use zircon_math::Vec3;
use zircon_scene::Scene;

use super::SceneViewportController;

impl SceneViewportController {
    pub(in crate::scene::viewport::controller) fn selected_world_position(
        scene: &Scene,
        selected: Option<u64>,
    ) -> Option<Vec3> {
        let selected = selected?;
        scene
            .world_transform(selected)
            .map(|transform| transform.translation)
            .or_else(|| {
                scene
                    .find_node(selected)
                    .map(|node| node.transform.translation)
            })
    }

    pub(in crate::scene::viewport::controller) fn select_node(
        &mut self,
        scene: &Scene,
        node_id: Option<u64>,
    ) -> bool {
        if self.selected_node() == node_id {
            return false;
        }
        self.set_selected_node(node_id);
        if let Some(target) = Self::selected_world_position(scene, node_id) {
            self.state.orbit_target = target;
        }
        true
    }
}

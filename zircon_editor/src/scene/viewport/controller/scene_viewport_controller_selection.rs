use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::Vec3;

use crate::scene::selection::SelectionMutation;

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

    pub(in crate::scene::viewport::controller) fn select_nodes(
        &mut self,
        scene: &Scene,
        node_ids: impl IntoIterator<Item = u64>,
        mutation: SelectionMutation,
    ) -> bool {
        let selectable = node_ids
            .into_iter()
            .filter(|node_id| scene.find_node(*node_id).is_some());
        let changed = self.state.selection.apply_active(selectable, mutation);
        if !changed {
            return false;
        }
        if let Some(target) =
            Self::selected_world_position(scene, self.state.selection.active_primary())
        {
            self.state.orbit_target = target;
            self.state.orbit_controller.set_target(target);
        }
        changed
    }
}

#[cfg(test)]
#[path = "tests/scene_viewport_controller_selection.rs"]
mod tests;

use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::Vec3;

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
        let unchanged = match node_id {
            Some(node_id) => {
                self.state.selection.active_primary() == Some(node_id)
                    && self.state.selection.active_items().len() == 1
            }
            None => self.state.selection.active_items().is_empty(),
        };
        if unchanged {
            return false;
        }
        match node_id {
            Some(node_id) => self.state.selection.select_only_active(node_id),
            None => self.state.selection.clear_active(),
        };
        if let Some(target) = Self::selected_world_position(scene, node_id) {
            self.state.orbit_target = target;
            self.state.orbit_controller.set_target(target);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::math::UVec2;

    #[test]
    fn selecting_the_current_primary_collapses_an_existing_multi_selection() {
        let scene = Scene::new();
        let entities = scene
            .nodes()
            .iter()
            .take(2)
            .map(|node| node.id)
            .collect::<Vec<_>>();
        let [primary, secondary] = entities.as_slice() else {
            panic!("default scene must contain at least two entities");
        };
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
        assert!(controller
            .selection_mut()
            .replace_active([*primary, *secondary], Some(*primary)));

        assert!(controller.select_node(&scene, Some(*primary)));

        assert_eq!(
            controller
                .selection()
                .active_items()
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            [*primary]
        );
    }
}

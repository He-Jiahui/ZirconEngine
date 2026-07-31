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
mod tests {
    use super::*;
    use crate::core::commands::{CommandEvalCtx, WhenClause};
    use crate::core::editor_message::SceneModeId;
    use crate::scene::modes::SceneModeActivation;
    use crate::scene::selection::WorldDomain;
    use crate::scene::viewport::TransformHandleKind;
    use crate::ui::binding::ViewportCommand;
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
        assert!(
            controller
                .selection_mut()
                .replace_active([*primary, *secondary], Some(*primary))
        );

        assert!(controller.select_nodes(&scene, [*primary], SelectionMutation::Replace));

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

    #[test]
    fn selection_rejects_invalid_overlay_owner_ids() {
        let scene = Scene::new();
        let valid = scene.nodes()[0].id;
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));

        assert!(controller.select_nodes(&scene, [valid, u64::MAX], SelectionMutation::Replace,));

        assert_eq!(
            controller
                .selection()
                .active_items()
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            [valid]
        );
        assert!(!controller.select_nodes(&scene, [u64::MAX], SelectionMutation::Extend,));
    }

    #[test]
    fn command_eval_projection_uses_mode_stack_and_active_domain_selection() {
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));

        let initial = controller.project_command_eval_ctx(CommandEvalCtx::interactive());
        assert!(WhenClause::SceneModeActive(SceneModeId::new("scene.select")).eval(&initial));
        assert!(!WhenClause::SelectionNonEmpty.eval(&initial));

        assert!(controller.selection_mut().select_only_active(11));
        let selected = controller.project_command_eval_ctx(CommandEvalCtx::interactive());
        assert!(WhenClause::SelectionNonEmpty.eval(&selected));

        controller
            .apply_command(
                None,
                &ViewportCommand::ActivateSceneMode(SceneModeActivation::Transform(
                    TransformHandleKind::Rotate,
                )),
            )
            .expect("scene mode selection does not persist settings");
        let rotated = controller.project_command_eval_ctx(CommandEvalCtx::interactive());
        assert!(WhenClause::SceneModeActive(SceneModeId::new("scene.transform")).eval(&rotated));
        assert!(WhenClause::SelectionNonEmpty.eval(&rotated));
        assert_eq!(
            controller.active_scene_mode(),
            SceneModeActivation::Transform(TransformHandleKind::Rotate)
        );

        assert!(
            controller
                .selection_mut()
                .set_active_domain(WorldDomain::Play)
        );
        let play = controller.project_command_eval_ctx(CommandEvalCtx::interactive());
        assert!(!WhenClause::SelectionNonEmpty.eval(&play));
    }
}

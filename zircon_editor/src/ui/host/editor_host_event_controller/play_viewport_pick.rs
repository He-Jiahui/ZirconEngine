use zircon_runtime::core::framework::scene::EntityId;

use crate::core::play::{PlayInstanceId, PlayKind, PlayMode, WorldDomain};
use crate::scene::selection::SelectionMutation;

use super::EditorHostEventController;

impl EditorHostEventController {
    /// Applies a renderer-owned SIE pick only to the matching active Play selection domain.
    ///
    /// `None` rejects a late or cross-session completion. `Some(false)` is a valid completion that
    /// happened to preserve the current selection.
    pub(crate) fn apply_play_viewport_pick_selection(
        &self,
        instance: PlayInstanceId,
        entity: Option<EntityId>,
        mutation: SelectionMutation,
    ) -> Option<bool> {
        if !matches!(
            self.play_sessions().mode_snapshot(),
            PlayMode::Playing {
                kind: PlayKind::Simulate
            }
        ) || self.play_sessions().attached_world_domain() != Some(WorldDomain::Play(instance))
        {
            return None;
        }

        let changed = {
            let mut shell = self.shell().lock();
            if !shell.state.is_playing()
                || shell.state.viewport_controller.selection().active_domain()
                    != WorldDomain::Play(instance)
            {
                return None;
            }
            shell
                .state
                .viewport_controller
                .selection_mut()
                .apply_active(entity, mutation)
        };
        if changed {
            self.play_gizmo
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .invalidate_projection();
        }
        Some(changed)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use zircon_runtime::core::CoreRuntime;
    use zircon_runtime::scene::DefaultLevelManager;
    use zircon_runtime_interface::math::UVec2;

    use crate::core::gateway::DetachedEditorRuntimeGateway;
    use crate::core::play::PlayKind;
    use crate::scene::selection::SelectionMutation;
    use crate::ui::host::EditorManager;
    use crate::ui::workbench::state::EditorState;

    use super::*;

    #[test]
    fn renderer_pick_changes_only_the_matching_play_selection_domain() {
        let core = CoreRuntime::new();
        let manager = Arc::new(EditorManager::new(&core.handle()).expect("editor manager"));
        let state = EditorState::with_default_selection_with_context(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
            Arc::clone(manager.context()),
        );
        let controller = EditorHostEventController::new(state, manager);
        let edit_primary = controller
            .shell()
            .lock()
            .state
            .viewport_controller
            .selection()
            .active_primary();
        let instance = controller
            .start_test_play_gateway(PlayKind::Simulate, Arc::new(DetachedEditorRuntimeGateway))
            .expect("play gateway");
        controller
            .shell()
            .lock()
            .state
            .enter_play_mode()
            .expect("editor play state");
        assert!(controller.sync_active_selection_world_domain());

        assert_eq!(
            controller.apply_play_viewport_pick_selection(
                instance,
                Some(991),
                SelectionMutation::Replace,
            ),
            Some(true)
        );
        assert_eq!(
            controller
                .shell()
                .lock()
                .state
                .viewport_controller
                .selection()
                .active_primary(),
            Some(991)
        );

        controller
            .shell()
            .lock()
            .state
            .exit_play_mode()
            .expect("restore edit selection");
        assert_eq!(
            controller
                .shell()
                .lock()
                .state
                .viewport_controller
                .selection()
                .active_primary(),
            edit_primary
        );
    }
}

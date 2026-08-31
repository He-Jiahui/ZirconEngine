use thiserror::Error;

use crate::core::editing::EditCommandError;
use crate::core::gateway::GatewaySessionIdentity;
use crate::core::play::{PlayInstanceId, WorldDomain};

use super::EditorHostEventController;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PlayWorldReplacementRetirementReport {
    replacement_epoch: u64,
    history_discarded: bool,
    selection_cleared: bool,
    hierarchy_cleared: bool,
    inspector_cleared: bool,
}

impl PlayWorldReplacementRetirementReport {
    pub(crate) const fn replacement_epoch(self) -> u64 {
        self.replacement_epoch
    }

    pub(crate) const fn history_discarded(self) -> bool {
        self.history_discarded
    }

    pub(crate) const fn selection_cleared(self) -> bool {
        self.selection_cleared
    }

    pub(crate) const fn hierarchy_cleared(self) -> bool {
        self.hierarchy_cleared
    }

    pub(crate) const fn inspector_cleared(self) -> bool {
        self.inspector_cleared
    }
}

#[derive(Debug, Error)]
pub(crate) enum PlayWorldReplacementRetirementError {
    #[error("runtime published the reserved zero Play world replacement epoch")]
    ZeroReplacementEpoch,
    #[error("Play world replacement belongs to a stale gateway")]
    StaleGateway,
    #[error(transparent)]
    History(#[from] EditCommandError),
}

impl EditorHostEventController {
    pub(crate) fn retire_replaced_play_world(
        &self,
        instance: PlayInstanceId,
        identity: &GatewaySessionIdentity,
        replacement_epoch: u64,
    ) -> Result<PlayWorldReplacementRetirementReport, PlayWorldReplacementRetirementError> {
        if replacement_epoch == 0 {
            return Err(PlayWorldReplacementRetirementError::ZeroReplacementEpoch);
        }
        let domain = WorldDomain::Play(instance);
        if self.play_sessions.attached_world_domain() != Some(domain)
            || self.world_gateway_identity(domain).as_ref() != Some(identity)
        {
            return Err(PlayWorldReplacementRetirementError::StaleGateway);
        }

        let history_discarded = self.context.transactions().discard_play_history(instance)?;
        self.retire_play_gizmo_local_state();
        let selection_cleared = self
            .shell
            .lock()
            .state
            .viewport_controller
            .selection_mut()
            .clear(domain);
        let hierarchy_cleared = self
            .play_hierarchy_projection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
        let inspector_cleared = self
            .play_inspector_projection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
        zircon_runtime::profile_counter!("editor", "play.world_replacement.retired_count", 1);

        Ok(PlayWorldReplacementRetirementReport {
            replacement_epoch,
            history_discarded,
            selection_cleared,
            hierarchy_cleared,
            inspector_cleared,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use zircon_runtime::core::CoreRuntime;
    use zircon_runtime::scene::{DefaultLevelManager, LevelSystem, NodeKind, World};
    use zircon_runtime_interface::math::UVec2;

    use crate::core::editing::{EditorCommand, HistoryContextId};
    use crate::core::gateway::InProcessGateway;
    use crate::core::play::PlayKind;
    use crate::scene::selection::SelectionMutation;
    use crate::ui::host::EditorManager;
    use crate::ui::workbench::state::EditorState;

    use super::*;

    fn active_simulate_controller() -> (EditorHostEventController, PlayInstanceId, LevelSystem) {
        let core = CoreRuntime::new();
        let manager = Arc::new(EditorManager::new(&core.handle()).expect("editor manager"));
        let state = EditorState::with_default_selection_with_context(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
            Arc::clone(manager.context()),
        );
        let controller = EditorHostEventController::new(state, manager);
        let play_level = DefaultLevelManager::default().create_default_level();
        let instance = controller
            .start_test_play_gateway(
                PlayKind::Simulate,
                Arc::new(InProcessGateway::for_authoring_level(play_level.clone())),
            )
            .expect("play gateway");
        controller
            .shell()
            .lock()
            .state
            .enter_play_mode()
            .expect("editor play state");
        assert!(controller.sync_active_selection_world_domain());
        (controller, instance, play_level)
    }

    #[test]
    fn replacement_retires_the_matching_play_selection() {
        let (controller, instance, play_level) = active_simulate_controller();
        let history = HistoryContextId::PlaySession(instance);
        let mut scope = controller
            .context()
            .transactions()
            .begin("create runtime node", history)
            .expect("attached Play history");
        scope
            .push(EditorCommand::create_node(NodeKind::Cube))
            .expect("runtime command");
        scope.commit().expect("runtime command commit");
        assert_eq!(
            controller.apply_play_viewport_pick_selection(
                instance,
                Some(991),
                SelectionMutation::Replace,
            ),
            Some(true)
        );
        let identity = controller
            .world_gateway_identity(WorldDomain::Play(instance))
            .expect("attached identity");
        play_level.replace_world_and_reset_runtime_state(World::empty());
        let replacement_epoch = play_level.capture_world_replacement_epoch();

        let report = controller
            .retire_replaced_play_world(instance, &identity, replacement_epoch)
            .expect("replacement retirement");

        assert_eq!(report.replacement_epoch(), replacement_epoch);
        assert!(report.history_discarded());
        assert!(report.selection_cleared());
        assert_eq!(
            controller
                .context()
                .transactions()
                .history_status(history)
                .unwrap()
                .len,
            0
        );
        assert!(controller
            .gateway_for(WorldDomain::Play(instance))
            .is_some());
        assert_eq!(
            controller
                .shell()
                .lock()
                .state
                .viewport_controller
                .selection()
                .active_primary(),
            None
        );
    }

    #[test]
    fn replacement_from_a_stale_gateway_cannot_retire_current_play_state() {
        let (controller, instance, _play_level) = active_simulate_controller();
        controller.apply_play_viewport_pick_selection(
            instance,
            Some(991),
            SelectionMutation::Replace,
        );
        let stale = GatewaySessionIdentity::new(
            99,
            zircon_runtime_interface::ZrRuntimeSessionHandle::new(99),
            99,
            None,
        );

        assert!(controller
            .retire_replaced_play_world(instance, &stale, 2)
            .is_err());
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
    }
}

use crate::core::play::{PlayInstanceId, WorldDomain};
use crate::scene::selection::SelectionModel;
use crate::ui::workbench::snapshot::EditorBridgeDiagnosticsSnapshot;
use crate::ui::workbench::startup::EditorSessionMode;
use zircon_runtime::plugin::BridgeDiagnosticsMatrix;

use super::no_project_open::no_project_open;
use super::{editor_state::EditorState, EditorStateOperationError};

#[derive(Clone, Debug)]
pub(crate) struct EditorPlaySession {
    selection: SelectionModel,
    gizmos_enabled: bool,
    session_mode_before_play: EditorSessionMode,
}

impl EditorPlaySession {
    fn capture(state: &EditorState) -> Self {
        Self {
            selection: state.viewport_controller.selection().clone(),
            gizmos_enabled: state.viewport_controller.settings().gizmos_enabled,
            session_mode_before_play: state.session_mode,
        }
    }
}

impl EditorState {
    pub fn is_playing(&self) -> bool {
        self.play_session.is_some()
    }

    pub fn enter_play_mode(&mut self) -> Result<bool, EditorStateOperationError> {
        if self.play_session.is_some() {
            self.set_status_line("Already in play mode");
            return Ok(false);
        }

        self.with_exclusive_scene_transition("enter editor play mode", |state, _transition| {
            let Some(()) = state.world.with_world(|_| ())? else {
                let error = no_project_open();
                state.set_status_line(error.to_string());
                return Err(error);
            };

            state.play_session = Some(EditorPlaySession::capture(state));
            state.viewport_controller.cancel_interaction();
            state.session_mode = EditorSessionMode::Playing;
            state.set_status_line("Entered play mode");
            Ok(true)
        })
    }

    pub(crate) fn activate_play_selection_domain(&mut self, instance: PlayInstanceId) -> bool {
        if self.play_session.is_none() {
            return false;
        }
        self.viewport_controller
            .selection_mut()
            .activate_play_domain(instance)
    }

    pub(crate) fn sync_selection_world_domain(&mut self, domain: WorldDomain) -> bool {
        match domain {
            WorldDomain::Edit => self
                .viewport_controller
                .selection_mut()
                .set_active_domain(WorldDomain::Edit),
            WorldDomain::Play(instance) => self.activate_play_selection_domain(instance),
        }
    }

    pub fn exit_play_mode(&mut self) -> Result<bool, EditorStateOperationError> {
        if !self.world.is_loaded() {
            let error = no_project_open();
            self.set_status_line(error.to_string());
            return Err(error);
        }

        if self.play_session.is_none() {
            self.set_status_line("Not in play mode");
            return Ok(false);
        }

        self.with_exclusive_scene_transition("exit editor play mode", |state, _transition| {
            let session = state
                .play_session
                .take()
                .ok_or(EditorStateOperationError::PlaySessionMissing)?;
            *state.viewport_controller.selection_mut() = session.selection;
            state.viewport_controller.settings_mut().gizmos_enabled = session.gizmos_enabled;
            state.session_mode = session.session_mode_before_play;
            state.sync_selection_state();
            state.set_status_line("Exited play mode and restored edit state");
            Ok(true)
        })
    }

    pub(crate) fn sync_bridge_diagnostics_matrix(
        &mut self,
        matrix: Option<&BridgeDiagnosticsMatrix>,
    ) {
        self.bridge_diagnostics = matrix
            .map(EditorBridgeDiagnosticsSnapshot::from_runtime_matrix)
            .unwrap_or_default();
    }
}

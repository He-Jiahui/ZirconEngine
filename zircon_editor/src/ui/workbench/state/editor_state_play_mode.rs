use crate::scene::selection::{SelectionModel, WorldDomain};
use crate::ui::workbench::snapshot::EditorBridgeDiagnosticsSnapshot;
use crate::ui::workbench::startup::EditorSessionMode;
use zircon_runtime::plugin::BridgeDiagnosticsMatrix;
use zircon_runtime::scene::Scene;

use super::editor_state::EditorState;
use super::no_project_open::no_project_open;

#[derive(Clone, Debug)]
pub(crate) struct EditorPlaySession {
    scene: Scene,
    selection: SelectionModel,
    gizmos_enabled: bool,
    session_mode_before_play: EditorSessionMode,
}

impl EditorPlaySession {
    fn capture(state: &EditorState, scene: Scene) -> Self {
        Self {
            scene,
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

    pub fn enter_play_mode(&mut self) -> Result<bool, String> {
        if self.play_session.is_some() {
            self.set_status_line("Already in play mode");
            return Ok(false);
        }

        self.with_exclusive_scene_transition("enter editor play mode", |state, _transition| {
            let Some(scene) = state.world.try_snapshot() else {
                let message = no_project_open();
                state.set_status_line(message.clone());
                return Err(message);
            };

            state.play_session = Some(EditorPlaySession::capture(state, scene));
            let edit_items = state
                .viewport_controller
                .selection()
                .items(WorldDomain::Edit)
                .iter()
                .copied()
                .collect::<Vec<_>>();
            let edit_primary = state
                .viewport_controller
                .selection()
                .primary(WorldDomain::Edit);
            let selection = state.viewport_controller.selection_mut();
            selection.replace(WorldDomain::Play, edit_items, edit_primary);
            selection.set_active_domain(WorldDomain::Play);
            state.viewport_controller.settings_mut().gizmos_enabled = false;
            state.session_mode = EditorSessionMode::Playing;
            state.set_status_line("Entered play mode");
            Ok(true)
        })
    }

    pub fn exit_play_mode(&mut self) -> Result<bool, String> {
        if !self.world.is_loaded() {
            let message = no_project_open();
            self.set_status_line(message.clone());
            return Err(message);
        }

        if self.play_session.is_none() {
            self.set_status_line("Not in play mode");
            return Ok(false);
        }

        self.with_exclusive_scene_transition("exit editor play mode", |state, _transition| {
            let session = state
                .play_session
                .take()
                .ok_or_else(|| "play session disappeared during exclusive exit".to_string())?;
            state
                .world
                .try_with_world_mut(|scene| *scene = session.scene)
                .ok_or_else(no_project_open)?;
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

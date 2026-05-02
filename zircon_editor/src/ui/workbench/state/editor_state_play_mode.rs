use crate::core::editing::history::EditorHistory;
use crate::ui::workbench::startup::EditorSessionMode;
use zircon_runtime::scene::{NodeId, Scene};

use super::editor_state::EditorState;
use super::no_project_open::no_project_open;

#[derive(Clone, Debug)]
pub(crate) struct EditorPlaySession {
    scene: Scene,
    selected_node: Option<NodeId>,
    history: EditorHistory,
    session_mode_before_play: EditorSessionMode,
}

impl EditorPlaySession {
    fn capture(state: &EditorState, scene: Scene) -> Self {
        Self {
            scene,
            selected_node: state.viewport_controller.selected_node(),
            history: state.history.clone(),
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
            self.status_line = "Already in play mode".to_string();
            return Ok(false);
        }

        let Some(scene) = self.world.try_snapshot() else {
            let message = no_project_open();
            self.status_line = message.clone();
            return Err(message);
        };

        self.play_session = Some(EditorPlaySession::capture(self, scene));
        self.session_mode = EditorSessionMode::Playing;
        self.history.clear();
        self.status_line = "Entered play mode".to_string();
        Ok(true)
    }

    pub fn exit_play_mode(&mut self) -> Result<bool, String> {
        if !self.world.is_loaded() {
            let message = no_project_open();
            self.status_line = message.clone();
            return Err(message);
        }

        let Some(session) = self.play_session.take() else {
            self.status_line = "Not in play mode".to_string();
            return Ok(false);
        };

        self.world
            .try_with_world_mut(|scene| *scene = session.scene)
            .ok_or_else(no_project_open)?;
        self.viewport_controller
            .set_selected_node(restored_selected_node(self, session.selected_node));
        self.history = session.history;
        self.session_mode = session.session_mode_before_play;
        self.sync_selection_state();
        self.status_line = "Exited play mode and restored edit state".to_string();
        Ok(true)
    }
}

fn restored_selected_node(state: &EditorState, selected_node: Option<NodeId>) -> Option<NodeId> {
    selected_node.filter(|node_id| {
        state
            .world
            .try_with_world(|scene| scene.find_node(*node_id).is_some())
            .unwrap_or(false)
    })
}

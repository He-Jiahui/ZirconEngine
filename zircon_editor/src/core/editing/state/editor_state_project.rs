use zircon_runtime::scene::LevelSystem;
use zircon_scene::Scene;

use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};

use super::editor_state::EditorState;

impl EditorState {
    pub fn replace_world(&mut self, world: LevelSystem, project_path: impl Into<String>) {
        self.world.replace(world);
        self.project_path = project_path.into();
        self.session_mode = EditorSessionMode::Project;
        self.project_open = true;
        self.welcome = WelcomePaneSnapshot::default();
        self.history.clear();
        self.clear_selected_node();
        let _ = self
            .world
            .try_with_world(|scene| self.viewport_controller.reset_from_scene(Some(scene)));
        self.sync_selection_state();
    }

    pub fn clear_project(&mut self, welcome: WelcomePaneSnapshot) {
        self.world.clear();
        self.project_path.clear();
        self.session_mode = EditorSessionMode::Welcome;
        self.project_open = false;
        self.welcome = welcome;
        self.history.clear();
        self.clear_selected_node();
        self.viewport_controller.reset_from_scene(None);
        self.sync_selection_state();
    }

    pub fn mark_project_open(&mut self) {
        self.session_mode = EditorSessionMode::Project;
        self.project_open = true;
    }

    pub fn set_session_mode(&mut self, session_mode: EditorSessionMode) {
        self.session_mode = session_mode;
    }

    pub fn set_welcome_snapshot(&mut self, welcome: WelcomePaneSnapshot) {
        self.welcome = welcome;
        if self.session_mode == EditorSessionMode::Welcome {
            self.status_line = self.welcome.status_message.clone();
        }
    }

    pub fn project_scene(&self) -> Option<Scene> {
        self.world.try_snapshot()
    }

    pub fn has_project_world(&self) -> bool {
        self.world.is_loaded()
    }
}

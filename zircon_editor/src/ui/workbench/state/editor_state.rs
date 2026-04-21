use crate::core::editing::history::EditorHistory;
use crate::scene::viewport::SceneViewportController;
use crate::ui::workbench::project::AssetWorkspaceState;
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};

use super::editor_world_slot::EditorWorldSlot;

/// Editor shell state shared between the UI host and the scene server.
#[derive(Debug)]
pub struct EditorState {
    pub(crate) world: EditorWorldSlot,
    pub(crate) viewport_controller: SceneViewportController,
    pub(crate) name_field: String,
    pub(crate) parent_field: String,
    pub(crate) transform_fields: [String; 3],
    pub(crate) mesh_import_path: String,
    pub(crate) asset_workspace: AssetWorkspaceState,
    pub(crate) project_path: String,
    pub(crate) session_mode: EditorSessionMode,
    pub(crate) welcome: WelcomePaneSnapshot,
    pub(crate) project_open: bool,
    pub(crate) status_line: String,
    pub(crate) history: EditorHistory,
}

impl EditorState {
    pub(crate) fn clear_selected_node(&mut self) {
        self.viewport_controller.set_selected_node(None);
    }
}

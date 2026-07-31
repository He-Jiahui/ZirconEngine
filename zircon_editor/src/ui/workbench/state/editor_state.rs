use std::collections::BTreeMap;
use std::sync::Arc;

use crate::core::context::EditorContext;
use crate::core::editing::authoring_world::EditorAuthoringWorld;
use crate::scene::viewport::SceneViewportController;
use crate::ui::workbench::project::AssetWorkspaceState;
use crate::ui::workbench::snapshot::{EditorBridgeDiagnosticsSnapshot, StatusTaskProgressSnapshot};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};

use super::editor_state_play_mode::EditorPlaySession;
use super::editor_state_viewport::GizmoTransactionCapture;
/// Editor shell state shared between the UI host and runtime scene inspection.
pub struct EditorState {
    pub(crate) context: Arc<EditorContext>,
    pub(crate) world: EditorAuthoringWorld,
    pub(crate) viewport_controller: SceneViewportController,
    pub(crate) name_field: String,
    pub(crate) parent_field: String,
    pub(crate) transform_fields: [String; 3],
    pub(crate) inspector_dynamic_fields: BTreeMap<String, String>,
    pub(crate) mesh_import_path: String,
    pub(crate) asset_workspace: AssetWorkspaceState,
    pub(crate) project_path: String,
    pub(crate) session_mode: EditorSessionMode,
    pub(crate) welcome: WelcomePaneSnapshot,
    pub(crate) project_open: bool,
    pub(crate) status_line: String,
    pub(crate) status_task_progress: Option<StatusTaskProgressSnapshot>,
    pub(crate) bridge_diagnostics: EditorBridgeDiagnosticsSnapshot,
    pub(in crate::ui::workbench) gizmo_transaction: Option<GizmoTransactionCapture>,
    pub(crate) play_session: Option<EditorPlaySession>,
}

impl EditorState {
    pub(crate) fn transactions(&self) -> &crate::core::editing::engine::EditorTransactionEngine {
        self.context.transactions()
    }
}

impl Drop for EditorState {
    fn drop(&mut self) {
        self.viewport_controller.shutdown_scene_modes();
    }
}

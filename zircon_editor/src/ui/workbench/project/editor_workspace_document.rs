use serde::{Deserialize, Serialize};

use super::project_editor_workspace::ProjectEditorWorkspace;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(in crate::ui::workbench::project) struct EditorWorkspaceDocument {
    pub(in crate::ui::workbench::project) format_version: u32,
    pub(in crate::ui::workbench::project) editor_workspace: ProjectEditorWorkspace,
}

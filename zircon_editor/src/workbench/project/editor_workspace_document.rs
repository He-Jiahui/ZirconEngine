use serde::{Deserialize, Serialize};

use super::project_editor_workspace::ProjectEditorWorkspace;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(in crate::workbench::project) struct EditorWorkspaceDocument {
    pub(in crate::workbench::project) format_version: u32,
    pub(in crate::workbench::project) editor_workspace: ProjectEditorWorkspace,
}

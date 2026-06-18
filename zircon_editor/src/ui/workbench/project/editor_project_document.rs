use std::path::PathBuf;

use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::scene::Scene;

use super::project_editor_workspace::ProjectEditorWorkspace;

#[derive(Clone, Debug, PartialEq)]
pub struct EditorProjectDocument {
    pub root_path: PathBuf,
    pub manifest: ProjectManifest,
    pub world: Scene,
    pub editor_workspace: Option<ProjectEditorWorkspace>,
    pub workspace_restore_diagnostics: Vec<EditorWorkspaceRestoreDiagnostic>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EditorWorkspaceRestoreDiagnostic {
    pub path: PathBuf,
    pub message: String,
}

impl EditorWorkspaceRestoreDiagnostic {
    pub(in crate::ui::workbench::project) fn new(
        path: impl Into<PathBuf>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

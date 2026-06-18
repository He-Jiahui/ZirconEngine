use std::fs;
use std::path::Path;

use zircon_runtime::scene::world::SceneProjectError;

use super::constants::EDITOR_PROJECT_FORMAT_VERSION;
use super::editor_project_document::EditorWorkspaceRestoreDiagnostic;
use super::editor_workspace_document::EditorWorkspaceDocument;
use super::project_editor_workspace::ProjectEditorWorkspace;
use super::workspace_document_path::workspace_document_path;

pub(in crate::ui::workbench::project) fn load_editor_workspace_with_diagnostics(
    root: &Path,
) -> (
    Option<ProjectEditorWorkspace>,
    Vec<EditorWorkspaceRestoreDiagnostic>,
) {
    let path = workspace_document_path(root);
    if !path.exists() {
        return (None, Vec::new());
    }
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            return (
                None,
                vec![EditorWorkspaceRestoreDiagnostic::new(
                    path,
                    error.to_string(),
                )],
            );
        }
    };
    match serde_json::from_str::<EditorWorkspaceDocument>(&source) {
        Ok(document) if document.format_version == EDITOR_PROJECT_FORMAT_VERSION => {
            (Some(document.editor_workspace), Vec::new())
        }
        Ok(document) => (
            None,
            vec![EditorWorkspaceRestoreDiagnostic::new(
                path,
                format!(
                    "unsupported editor workspace format version {}",
                    document.format_version
                ),
            )],
        ),
        Err(error) => (
            None,
            vec![EditorWorkspaceRestoreDiagnostic::new(
                path,
                error.to_string(),
            )],
        ),
    }
}

pub(in crate::ui::workbench::project) fn save_editor_workspace(
    root: &Path,
    editor_workspace: Option<&ProjectEditorWorkspace>,
) -> Result<(), SceneProjectError> {
    let path = workspace_document_path(root);
    if let Some(workspace) = editor_workspace {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let document = EditorWorkspaceDocument {
            format_version: EDITOR_PROJECT_FORMAT_VERSION,
            editor_workspace: workspace.clone(),
        };
        fs::write(path, serde_json::to_string_pretty(&document)?)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

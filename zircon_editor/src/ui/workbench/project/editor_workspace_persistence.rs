use std::fs;
use std::path::Path;

use zircon_scene::SceneProjectError;

use super::constants::EDITOR_PROJECT_FORMAT_VERSION;
use super::editor_workspace_document::EditorWorkspaceDocument;
use super::project_editor_workspace::ProjectEditorWorkspace;
use super::workspace_document_path::workspace_document_path;

pub(in crate::ui::workbench::project) fn load_editor_workspace(
    root: &Path,
) -> Result<Option<ProjectEditorWorkspace>, SceneProjectError> {
    let path = workspace_document_path(root);
    if !path.exists() {
        return Ok(None);
    }
    let document = serde_json::from_str::<EditorWorkspaceDocument>(&fs::read_to_string(path)?)?;
    Ok(Some(document.editor_workspace))
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

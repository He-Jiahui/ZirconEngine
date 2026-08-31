use std::fs;
use std::io;
use std::path::Path;

use zircon_runtime::core::resource::io::atomic_write;
use zircon_runtime::scene::world::SceneProjectError;

#[cfg(test)]
#[path = "editor_workspace_persistence/borrowed_save_tests.rs"]
mod borrowed_save_tests;

use super::editor_project_document::EditorWorkspaceRestoreDiagnostic;
use super::editor_workspace_document::{
    decode_editor_workspace_document, encode_editor_workspace_document,
};
use super::project_editor_workspace::ProjectEditorWorkspace;
use super::workspace_document_path::workspace_document_path;

#[derive(Debug)]
pub(in crate::ui::workbench::project) enum PersistedWorkspaceSnapshot {
    Missing,
    File(Vec<u8>),
}

pub(in crate::ui::workbench::project) fn capture_editor_workspace(
    root: &Path,
) -> Result<PersistedWorkspaceSnapshot, SceneProjectError> {
    let path = workspace_document_path(root);
    match fs::read(path) {
        Ok(bytes) => Ok(PersistedWorkspaceSnapshot::File(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(PersistedWorkspaceSnapshot::Missing)
        }
        Err(error) => Err(error.into()),
    }
}

pub(in crate::ui::workbench::project) fn restore_editor_workspace(
    root: &Path,
    snapshot: PersistedWorkspaceSnapshot,
) -> Result<(), SceneProjectError> {
    let path = workspace_document_path(root);
    match snapshot {
        PersistedWorkspaceSnapshot::Missing => match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        },
        PersistedWorkspaceSnapshot::File(bytes) => Ok(atomic_write(&path, &bytes)?),
    }
}

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
    let source = match fs::read(&path) {
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
    match decode_editor_workspace_document(&source) {
        Ok(workspace) => (Some(workspace), Vec::new()),
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
        let serialized = encode_editor_workspace_document(workspace).map_err(io::Error::other)?;
        atomic_write(&path, serialized.as_bytes())?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        capture_editor_workspace, restore_editor_workspace, workspace_document_path,
        PersistedWorkspaceSnapshot,
    };

    #[test]
    fn missing_workspace_snapshot_removes_a_workspace_written_before_scene_rollback() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon_editor_missing_workspace_{}_{}",
            std::process::id(),
            unique
        ));
        fs::create_dir_all(&root).unwrap();

        let snapshot = capture_editor_workspace(&root).unwrap();
        assert!(matches!(&snapshot, PersistedWorkspaceSnapshot::Missing));

        let workspace_path = workspace_document_path(&root);
        fs::create_dir_all(workspace_path.parent().unwrap()).unwrap();
        fs::write(&workspace_path, b"workspace written before scene failure").unwrap();

        restore_editor_workspace(&root, snapshot).unwrap();
        assert!(
            !workspace_path.exists(),
            "rolling back a missing workspace snapshot must remove the newly written workspace"
        );

        fs::remove_dir_all(root).unwrap();
    }
}

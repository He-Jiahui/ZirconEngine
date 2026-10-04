use std::fs;
use std::io;
use std::path::Path;

use zircon_runtime::core::resource::io::atomic_write;
use zircon_runtime::scene::world::SceneProjectError;

#[cfg(test)]
#[path = "editor_workspace_persistence/tests/borrowed_save_tests.rs"]
mod borrowed_save_tests;

use super::editor_project_document::EditorWorkspaceRestoreDiagnostic;
use super::editor_workspace_document::{
    decode_editor_workspace_document, encode_editor_workspace_document,
};
use super::project_editor_workspace::ProjectEditorWorkspace;
use super::workspace_document_path::workspace_document_path;

#[derive(Debug)]
/// 场景保存失败的补偿材料；保留原文件字节及原本不存在的区别。
pub(in crate::ui::workbench::project) enum PersistedWorkspaceSnapshot {
    Missing,
    File(Vec<u8>),
}

/// 在辅助workspace改写前捕获原始字节，补偿时不应重新编码旧文档。
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

/// 恢复精确旧字节或删除新文件；补偿失败必须传播，不能报告已恢复。
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

/// 辅助workspace无效允许默认布局继续，错误路径仍携显式诊断。
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

/// 写版本化辅助文档；None表达不保留项目workspace，不是忽略当前保存。
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
#[path = "tests/editor_workspace_persistence.rs"]
mod tests;

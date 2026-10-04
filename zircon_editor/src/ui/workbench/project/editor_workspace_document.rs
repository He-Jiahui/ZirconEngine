use serde::{Deserialize, Serialize};
use serde_json::Value;
use zircon_runtime_interface::serialization::{
    load_versioned, write_versioned_text, Format, LoadError, MigrateError, MigrationChain,
    MigrationStep, SchemaId, VersionedSchema, WriteError,
};

use super::project_editor_workspace::ProjectEditorWorkspace;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::ui::workbench::project) struct EditorWorkspaceDocument {
    pub(in crate::ui::workbench::project) editor_workspace: ProjectEditorWorkspace,
}

#[derive(Serialize)]
struct EditorWorkspaceDocumentRef<'workspace> {
    editor_workspace: &'workspace ProjectEditorWorkspace,
}

/// 借用当前workspace编码为版本壳，避免为辅助保存复制整棵布局。
pub(super) fn encode_editor_workspace_document(
    workspace: &ProjectEditorWorkspace,
) -> Result<String, WriteError> {
    write_versioned_text(&EditorWorkspaceDocumentRef {
        editor_workspace: workspace,
    })
}

/// 只解版本协议；视图注册与身份冲突仍须由恢复入口验证。
pub(super) fn decode_editor_workspace_document(
    source: &[u8],
) -> Result<ProjectEditorWorkspace, LoadError> {
    Ok(
        load_versioned::<EditorWorkspaceDocument>(source, Format::Text)?
            .value
            .editor_workspace,
    )
}

impl VersionedSchema for EditorWorkspaceDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.editor.workbench.project-workspace");
    const VERSION: u32 = 2;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<EditorWorkspaceDocument> = MigrationChain::new(&[
            MigrationStep::new(0, reject_legacy_workspace_document),
            MigrationStep::new(1, add_scene_viewport_sessions),
        ]);
        &MIGRATIONS
    }
}

impl<'workspace> VersionedSchema for EditorWorkspaceDocumentRef<'workspace> {
    const SCHEMA: SchemaId = EditorWorkspaceDocument::SCHEMA;
    const VERSION: u32 = EditorWorkspaceDocument::VERSION;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<EditorWorkspaceDocumentRef<'static>> =
            MigrationChain::new(&[
                MigrationStep::new(0, reject_legacy_workspace_document),
                MigrationStep::new(1, add_scene_viewport_sessions),
            ]);
        &MIGRATIONS
    }
}

fn add_scene_viewport_sessions(mut value: Value) -> Result<Value, MigrateError> {
    let workspace = value
        .as_object_mut()
        .and_then(|document| document.get_mut("editor_workspace"))
        .and_then(Value::as_object_mut)
        .ok_or_else(|| {
            MigrateError::invalid_payload(
                "version-one editor workspace payload must contain an editor_workspace object",
            )
        })?;
    workspace
        .entry("scene_viewport_sessions")
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    Ok(value)
}

/// 明确拒绝未版本化的旧载荷，不能自动把不明旧布局当作当前协议。
fn reject_legacy_workspace_document(_value: Value) -> Result<Value, MigrateError> {
    Err(MigrateError::invalid_payload(
        "unversioned editor workspace documents are retired",
    ))
}

#[cfg(test)]
#[path = "tests/editor_workspace_document.rs"]
mod tests;

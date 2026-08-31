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

pub(super) fn encode_editor_workspace_document(
    workspace: &ProjectEditorWorkspace,
) -> Result<String, WriteError> {
    write_versioned_text(&EditorWorkspaceDocumentRef {
        editor_workspace: workspace,
    })
}

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
    const VERSION: u32 = 1;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<EditorWorkspaceDocument> =
            MigrationChain::new(&[MigrationStep::new(0, reject_legacy_workspace_document)]);
        &MIGRATIONS
    }
}

impl<'workspace> VersionedSchema for EditorWorkspaceDocumentRef<'workspace> {
    const SCHEMA: SchemaId = EditorWorkspaceDocument::SCHEMA;
    const VERSION: u32 = EditorWorkspaceDocument::VERSION;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<EditorWorkspaceDocumentRef<'static>> =
            MigrationChain::new(&[MigrationStep::new(0, reject_legacy_workspace_document)]);
        &MIGRATIONS
    }
}

fn reject_legacy_workspace_document(_value: Value) -> Result<Value, MigrateError> {
    Err(MigrateError::invalid_payload(
        "unversioned editor workspace documents are retired",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::workbench::layout::WorkbenchLayout;

    fn workspace() -> ProjectEditorWorkspace {
        ProjectEditorWorkspace {
            workbench: WorkbenchLayout::default(),
            open_view_instances: Vec::new(),
            focused_view: None,
            active_drawers: Vec::new(),
        }
    }

    #[test]
    fn project_workspace_uses_the_current_version_shell_and_roundtrips() {
        let workspace = workspace();

        let encoded = encode_editor_workspace_document(&workspace).unwrap();

        assert!(encoded.contains(EditorWorkspaceDocument::SCHEMA.as_str()));
        assert_eq!(
            decode_editor_workspace_document(encoded.as_bytes()).unwrap(),
            workspace
        );
    }

    #[test]
    fn unversioned_project_workspace_is_rejected() {
        let legacy = serde_json::to_vec(&EditorWorkspaceDocument {
            editor_workspace: workspace(),
        })
        .unwrap();

        assert!(matches!(
            decode_editor_workspace_document(&legacy),
            Err(LoadError::MissingTextEnvelope { .. })
        ));
    }
}

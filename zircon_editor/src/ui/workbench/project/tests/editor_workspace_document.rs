use super::*;
use crate::ui::workbench::layout::WorkbenchLayout;

fn workspace() -> ProjectEditorWorkspace {
    ProjectEditorWorkspace {
        workbench: WorkbenchLayout::default(),
        open_view_instances: Vec::new(),
        focused_view: None,
        active_drawers: Vec::new(),
        scene_viewport_sessions: Default::default(),
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
fn version_one_project_workspace_migrates_an_empty_per_scene_map() {
    let workspace = workspace();
    let encoded = encode_editor_workspace_document(&workspace).unwrap();
    let mut legacy: Value = serde_json::from_str(&encoded).unwrap();
    legacy["header"]["schema_version"] = Value::from(1);
    legacy["payload"]["editor_workspace"]
        .as_object_mut()
        .unwrap()
        .remove("scene_viewport_sessions");
    let legacy_bytes = serde_json::to_vec(&legacy).unwrap();

    let restored = decode_editor_workspace_document(&legacy_bytes).unwrap();

    assert_eq!(restored, workspace);
    assert!(restored.scene_viewport_sessions.is_empty());
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

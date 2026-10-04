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

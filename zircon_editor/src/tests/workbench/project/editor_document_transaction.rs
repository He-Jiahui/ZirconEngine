use std::fs;
use std::path::Path;

use zircon_runtime::asset::project::{
    EditorDocumentCommitFault, EditorDocumentScope, ProjectManager,
};

use crate::ui::workbench::layout::{DocumentNode, MainPageId, TabStackLayout, WorkbenchLayout};
use crate::ui::workbench::project::{EditorProjectDocument, ProjectEditorWorkspace};
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};

use super::document_roundtrip::{create_renderable_project, unique_mvp_project_root};

#[test]
fn editor_project_document_reopens_distinct_pairs_after_workspace_and_none_faults() {
    let root = unique_mvp_project_root("distinct-document-transaction-pairs");
    create_renderable_project(&root);
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let initial = EditorProjectDocument::load_from_project_for_tests(&project).unwrap();
    let scene_uri = project.manifest().default_scene.clone();
    let scene_path = project.source_path_for_uri(&scene_uri).unwrap();
    let workspace_path = root.join(".zircon").join("editor-workspace.json");
    let old_scene = initial.world.clone();
    let mut new_scene = old_scene.clone();
    let cube = new_scene
        .nodes()
        .iter()
        .find(|node| node.name == "Cube")
        .expect("renderable project must contain the template Cube")
        .clone();
    let mut changed_transform = cube.transform.clone();
    changed_transform.translation.x += 9.25;
    assert!(new_scene
        .update_transform(cube.id, changed_transform)
        .unwrap());
    let old_workspace = transaction_workspace("serialized-old");
    let new_workspace = transaction_workspace("serialized-new");

    EditorProjectDocument::save_scene_to_project(
        &project,
        &scene_uri,
        &old_scene,
        Some(&old_workspace),
    )
    .unwrap();
    let old_scene_bytes = fs::read(&scene_path).unwrap();
    let old_workspace_bytes = fs::read(&workspace_path).unwrap();
    let previous_pair = (
        old_scene.clone(),
        old_workspace.clone(),
        old_scene_bytes.clone(),
        old_workspace_bytes.clone(),
    );
    EditorProjectDocument::save_scene_to_project(
        &project,
        &scene_uri,
        &new_scene,
        Some(&new_workspace),
    )
    .unwrap();
    let new_scene_bytes = fs::read(&scene_path).unwrap();
    let new_workspace_bytes = fs::read(&workspace_path).unwrap();
    assert_ne!(old_scene_bytes, new_scene_bytes);
    assert_ne!(old_workspace_bytes, new_workspace_bytes);

    // Every interruption starts from a valid old pair and reopens through the actual decoder.
    EditorProjectDocument::save_scene_to_project(
        &project,
        &scene_uri,
        &old_scene,
        Some(&old_workspace),
    )
    .unwrap();
    let generation = project.catalog_input_generation().sequence();
    let scope = project
        .scoped_editor_document(&EditorDocumentScope::new(&root, generation))
        .unwrap();
    assert!(scope
        .commit_scene_workspace_with_fault_for_test(
            &scene_uri,
            &new_scene,
            &new_workspace_bytes,
            EditorDocumentCommitFault::CrashAfterTargetReplace(0),
        )
        .is_err());
    assert_fault_stage(
        &root,
        &scene_path,
        &new_scene_bytes,
        &workspace_path,
        Some(&old_workspace_bytes),
        false,
    );
    drop(scope);
    drop(project);

    let mut reopened = ProjectManager::open(&root).unwrap();
    reopened.scan_and_import().unwrap();
    let loaded_old = EditorProjectDocument::load_from_project_for_tests(&reopened).unwrap();
    assert_eq!(fs::read(&scene_path).unwrap(), previous_pair.2);
    assert_eq!(fs::read(&workspace_path).unwrap(), previous_pair.3);
    assert_eq!(loaded_old.world.nodes(), previous_pair.0.nodes());
    assert_eq!(loaded_old.editor_workspace, Some(previous_pair.1.clone()));
    assert_journal_is_empty(&root);
    drop(loaded_old);

    EditorProjectDocument::save_scene_to_project(
        &reopened,
        &scene_uri,
        &new_scene,
        Some(&new_workspace),
    )
    .unwrap();
    drop(reopened);
    let mut committed = ProjectManager::open(&root).unwrap();
    committed.scan_and_import().unwrap();
    let loaded_new = EditorProjectDocument::load_from_project_for_tests(&committed).unwrap();
    assert_eq!(fs::read(&scene_path).unwrap(), new_scene_bytes);
    assert_eq!(fs::read(&workspace_path).unwrap(), new_workspace_bytes);
    assert_eq!(loaded_new.world.nodes(), new_scene.nodes());
    assert_eq!(loaded_new.editor_workspace, Some(new_workspace.clone()));
    drop(loaded_new);

    for fault in [
        EditorDocumentCommitFault::CrashAfterTargetReplace(0),
        EditorDocumentCommitFault::CrashAfterRetiredDelete(0),
    ] {
        EditorProjectDocument::save_scene_to_project(
            &committed,
            &scene_uri,
            &old_scene,
            Some(&old_workspace),
        )
        .unwrap();
        let generation = committed.catalog_input_generation().sequence();
        let scope = committed
            .scoped_editor_document(&EditorDocumentScope::new(&root, generation))
            .unwrap();
        assert!(scope
            .commit_scene_without_workspace_with_fault_for_test(&scene_uri, &new_scene, fault)
            .is_err());
        assert_fault_stage(
            &root,
            &scene_path,
            &new_scene_bytes,
            &workspace_path,
            if matches!(fault, EditorDocumentCommitFault::CrashAfterTargetReplace(_)) {
                Some(&old_workspace_bytes)
            } else {
                None
            },
            true,
        );
        drop(scope);
        drop(committed);

        let mut recovered = ProjectManager::open(&root).unwrap();
        recovered.scan_and_import().unwrap();
        let loaded_old = EditorProjectDocument::load_from_project_for_tests(&recovered).unwrap();
        assert_eq!(fs::read(&scene_path).unwrap(), previous_pair.2);
        assert_eq!(fs::read(&workspace_path).unwrap(), previous_pair.3);
        assert_eq!(loaded_old.world.nodes(), previous_pair.0.nodes());
        assert_eq!(loaded_old.editor_workspace, Some(previous_pair.1.clone()));
        assert_journal_is_empty(&root);
        drop(loaded_old);
        committed = recovered;
    }

    EditorProjectDocument::save_scene_to_project(&committed, &scene_uri, &new_scene, None).unwrap();
    assert!(!workspace_path.exists());
    drop(committed);
    let mut absent = ProjectManager::open(&root).unwrap();
    absent.scan_and_import().unwrap();
    let loaded_new = EditorProjectDocument::load_from_project_for_tests(&absent).unwrap();
    assert_eq!(loaded_new.world.nodes(), new_scene.nodes());
    assert!(loaded_new.editor_workspace.is_none());
    let new_none_scene_bytes = fs::read(&scene_path).unwrap();
    drop(loaded_new);

    let generation = absent.catalog_input_generation().sequence();
    let scope = absent
        .scoped_editor_document(&EditorDocumentScope::new(&root, generation))
        .unwrap();
    assert!(scope
        .commit_scene_without_workspace_with_fault_for_test(
            &scene_uri,
            &old_scene,
            EditorDocumentCommitFault::CrashAfterTargetReplace(0),
        )
        .is_err());
    assert_fault_stage(
        &root,
        &scene_path,
        &old_scene_bytes,
        &workspace_path,
        None,
        false,
    );
    drop(scope);
    drop(absent);
    let mut final_project = ProjectManager::open(&root).unwrap();
    final_project.scan_and_import().unwrap();
    let loaded_absent = EditorProjectDocument::load_from_project_for_tests(&final_project).unwrap();
    assert_eq!(fs::read(&scene_path).unwrap(), new_none_scene_bytes);
    assert!(loaded_absent.editor_workspace.is_none());
    assert!(!workspace_path.exists());
    assert_journal_is_empty(&root);
    drop(loaded_absent);
    drop(final_project);
    let mut reopened_final = ProjectManager::open(&root).unwrap();
    reopened_final.scan_and_import().unwrap();
    let loaded_old = EditorProjectDocument::load_from_project_for_tests(&reopened_final).unwrap();
    assert_eq!(loaded_old.world.nodes(), new_scene.nodes());
    assert!(loaded_old.editor_workspace.is_none());
    assert!(!workspace_path.exists());
    drop(loaded_old);
    drop(reopened_final);
    let _ = fs::remove_dir_all(&root);
}

fn assert_fault_stage(
    root: &Path,
    scene_path: &Path,
    expected_scene_bytes: &[u8],
    workspace_path: &Path,
    expected_workspace_bytes: Option<&[u8]>,
    expect_workspace_retirement: bool,
) {
    let journal = root.join(".zircon").join("editor-document");
    assert!(
        journal.is_dir(),
        "fault injection must leave a recovery journal"
    );
    let journal_path = fs::read_dir(&journal)
        .expect("fault journal should be readable")
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().and_then(|value| value.to_str()) == Some("zrjournal"))
        .expect("fault journal should contain one framed transaction");
    let journal_bytes = fs::read(&journal_path).unwrap();
    let journal_text = String::from_utf8_lossy(&journal_bytes);
    assert!(
        journal_text.contains(
            scene_path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("scene target file name")
        ),
        "fault journal must retain the Scene target identity"
    );
    if expect_workspace_retirement {
        assert!(
            journal_text.contains(
                workspace_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .expect("workspace target file name")
            ),
            "None workspace publication must retain the retired workspace target in its journal"
        );
    }
    assert_eq!(fs::read(scene_path).unwrap(), expected_scene_bytes);
    match expected_workspace_bytes {
        Some(expected) => assert_eq!(fs::read(workspace_path).unwrap(), expected),
        None => assert!(!workspace_path.exists()),
    }
}

fn transaction_workspace(focused_view: &str) -> ProjectEditorWorkspace {
    let instance_id = ViewInstanceId::new(focused_view);
    let mut workbench = WorkbenchLayout::default();
    *workbench
        .content_workspace_for_page_mut(&MainPageId::workbench())
        .expect("default workbench page") = DocumentNode::tabs(TabStackLayout {
        tabs: vec![instance_id.clone()],
        active_tab: Some(instance_id.clone()),
    });
    ProjectEditorWorkspace {
        workbench,
        open_view_instances: vec![ViewInstance {
            instance_id: instance_id.clone(),
            descriptor_id: ViewDescriptorId::new("editor.scene"),
            title: focused_view.to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Document(MainPageId::workbench(), Vec::new()),
        }],
        focused_view: Some(instance_id),
        active_drawers: Vec::new(),
        scene_viewport_sessions: std::collections::BTreeMap::new(),
    }
}

fn assert_journal_is_empty(root: &Path) {
    let journal = root.join(".zircon").join("editor-document");
    if journal.exists() {
        assert!(
            fs::read_dir(journal)
                .expect("editor document journal should be readable")
                .next()
                .is_none(),
            "ProjectManager reopen must consume the editor document recovery journal"
        );
    }
}

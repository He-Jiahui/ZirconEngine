use std::fs;

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::watch::{AssetChange, AssetChangeKind};

use crate::core::asset::EditorAssetFolderOperation as Operation;

use super::super::project_mutation_tests::{exercise, uri, SOURCE};
use super::DefaultEditorAssetManager;

#[test]
fn folder_operations_require_the_runtime_asset_owner() {
    let error = DefaultEditorAssetManager::new()
        .submit_project_folder_mutation(Operation::Create, uri("res://new-folder"), None)
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("requires the runtime asset manager"));
}

#[test]
fn folder_create_rename_delete_publish_empty_folders_and_survive_reopen() {
    exercise(|scope| {
        let before = scope.editor.catalog_snapshot_record();
        let created = scope
            .editor
            .submit_project_folder_mutation(Operation::Create, uri("res://new-folder"), None)
            .unwrap()
            .wait_until(scope.deadline())
            .unwrap()
            .unwrap();
        assert_eq!(created.operation(), Operation::Create);
        assert_eq!(created.source(), &uri("res://new-folder"));
        assert!(scope.assets_root().join("new-folder").is_dir());
        let after_create = scope.editor.catalog_snapshot_record();
        assert!(after_create.folder("res://new-folder").is_some());
        assert_eq!(after_create.assets.len(), before.assets.len());
        assert!(after_create.catalog_revision > before.catalog_revision);
        let reopened = DefaultEditorAssetManager::new();
        reopened
            .sync_from_project(ProjectManager::open(scope.root()).unwrap())
            .unwrap();
        assert!(reopened
            .catalog_snapshot_record()
            .folder("res://new-folder")
            .is_some());

        scope
            .editor
            .submit_project_folder_mutation(
                Operation::Rename,
                uri("res://new-folder"),
                Some(uri("res://renamed-folder")),
            )
            .unwrap()
            .wait_until(scope.deadline())
            .unwrap()
            .unwrap();
        assert!(!scope.assets_root().join("new-folder").exists());
        assert!(scope.assets_root().join("renamed-folder").is_dir());
        let after_rename = scope.editor.catalog_snapshot_record();
        assert!(after_rename.folder("res://new-folder").is_none());
        assert!(after_rename.folder("res://renamed-folder").is_some());

        scope
            .editor
            .submit_project_folder_mutation(Operation::Delete, uri("res://renamed-folder"), None)
            .unwrap()
            .wait_until(scope.deadline())
            .unwrap()
            .unwrap();
        assert!(!scope.assets_root().join("renamed-folder").exists());
        assert!(scope
            .editor
            .catalog_snapshot_record()
            .folder("res://renamed-folder")
            .is_none());
        assert!(scope
            .editor
            .catalog_snapshot_record()
            .asset_by_locator(SOURCE)
            .is_some());
    });
}

#[test]
fn folder_rename_preserves_registered_asset_identity_and_sidecar() {
    exercise(|scope| {
        let uuid = scope
            .snapshot()
            .asset_registry()
            .entry_by_path(&uri(SOURCE))
            .unwrap()
            .uuid();
        let original = fs::read(scope.assets_root().join("data/item.u09data")).unwrap();
        let result = scope
            .editor
            .submit_project_folder_mutation(
                Operation::Rename,
                uri("res://data"),
                Some(uri("res://renamed")),
            )
            .unwrap()
            .wait_until(scope.deadline())
            .unwrap()
            .unwrap();
        assert!(result
            .statuses()
            .iter()
            .any(|status| status.uri == "res://renamed/item.u09data"));
        assert_eq!(
            fs::read(scope.assets_root().join("renamed/item.u09data")).unwrap(),
            original
        );
        assert!(scope
            .assets_root()
            .join("renamed/item.u09data.zmeta")
            .is_file());
        assert!(!scope.assets_root().join("data").exists());
        assert_eq!(
            scope
                .snapshot()
                .asset_registry()
                .entry_by_uuid(uuid)
                .unwrap()
                .path(),
            &uri("res://renamed/item.u09data")
        );
        assert_eq!(
            scope
                .editor
                .catalog_snapshot_record()
                .asset(&uuid.to_string())
                .unwrap()
                .locator,
            "res://renamed/item.u09data"
        );
    });
}

#[test]
fn folder_delete_refuses_populated_directory_and_preserves_catalog() {
    exercise(|scope| {
        let before = scope.editor.catalog_snapshot_record();
        let result = scope
            .editor
            .submit_project_folder_mutation(Operation::Delete, uri("res://data"), None)
            .unwrap()
            .wait_until(scope.deadline())
            .unwrap();
        assert!(result.is_err());
        assert!(scope.assets_root().join("data/item.u09data").is_file());
        assert_eq!(
            scope.editor.catalog_snapshot_record().catalog_revision,
            before.catalog_revision
        );
    });
}

#[test]
fn folder_rename_refuses_changed_source_until_reimport_and_preserves_dirty_state() {
    exercise(|scope| {
        let uuid = scope
            .snapshot()
            .asset_registry()
            .entry_by_path(&uri(SOURCE))
            .unwrap()
            .uuid();
        fs::write(
            scope.assets_root().join("data/item.u09data"),
            "{\"version\":2}",
        )
        .unwrap();
        scope
            .editor
            .project_runtime_asset_changes(&[std::sync::Arc::new(AssetChange::new(
                AssetChangeKind::Modified,
                uri(SOURCE),
                None,
            ))]);
        let error = scope
            .editor
            .submit_project_folder_mutation(
                Operation::Rename,
                uri("res://data"),
                Some(uri("res://renamed")),
            )
            .unwrap()
            .wait_until(scope.deadline())
            .unwrap()
            .unwrap_err();
        assert!(error.to_string().contains("current imported generation"));
        assert!(scope.assets_root().join("data/item.u09data").is_file());
        assert!(
            scope
                .editor
                .catalog_snapshot_record()
                .asset_by_locator(SOURCE)
                .unwrap()
                .dirty
        );
        scope
            .editor
            .submit_project_asset_reimport(uuid)
            .unwrap()
            .wait_until(scope.deadline())
            .unwrap()
            .unwrap();
        scope
            .editor
            .submit_project_folder_mutation(
                Operation::Rename,
                uri("res://data"),
                Some(uri("res://renamed")),
            )
            .unwrap()
            .wait_until(scope.deadline())
            .unwrap()
            .unwrap();
        assert!(scope.assets_root().join("renamed/item.u09data").is_file());
        assert!(
            scope
                .editor
                .catalog_snapshot_record()
                .asset(&uuid.to_string())
                .unwrap()
                .dirty
        );
    });
}

#[test]
fn queued_folder_create_refuses_a_reopened_project_activation() {
    super::super::project_mutation_tests::exercise_with_reopen(|scope| {
        let gate = scope.block_mutation_queue();
        let ticket = scope
            .editor
            .submit_project_folder_mutation(Operation::Create, uri("res://old-request"), None)
            .unwrap();
        scope.reopen();
        drop(gate);
        let error = ticket.wait_until(scope.deadline()).unwrap().unwrap_err();
        assert!(error.to_string().contains("inactive"));
        assert!(!scope.assets_root().join("old-request").exists());
        assert!(scope
            .editor
            .catalog_snapshot_record()
            .folder("res://old-request")
            .is_none());
    });
}

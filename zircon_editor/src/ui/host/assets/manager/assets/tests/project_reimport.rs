use std::sync::atomic::Ordering;

use zircon_runtime::core::manager::resolve_manager_service;

use super::super::project_mutation_tests::{exercise, uri, SOURCE};
use super::DefaultEditorAssetManager;

#[test]
fn reimport_requires_the_runtime_asset_owner() {
    let error = DefaultEditorAssetManager::new()
        .submit_project_asset_reimport(zircon_runtime::asset::AssetUuid::new())
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("requires the runtime asset manager"));
}

#[test]
fn reimport_reruns_unchanged_source_and_preserves_uuid_and_catalog() {
    exercise(|scope| {
        let before_project = scope.snapshot();
        let uuid = before_project
            .asset_registry()
            .entry_by_path(&uri(SOURCE))
            .unwrap()
            .uuid();
        let before_calls = scope.calls.load(Ordering::SeqCst);
        let before = scope.editor.catalog_snapshot_record();
        let result = scope
            .editor
            .submit_project_asset_reimport(uuid)
            .unwrap()
            .wait_until(scope.deadline())
            .unwrap()
            .unwrap();
        assert_eq!(result.target_uuid(), &uuid);
        assert_eq!(result.source(), &uri(SOURCE));
        assert!(result.status().unwrap().imported);
        assert_eq!(scope.calls.load(Ordering::SeqCst), before_calls + 1);
        let project = scope.snapshot();
        assert_eq!(
            project
                .asset_registry()
                .entry_by_path(&uri(SOURCE))
                .unwrap()
                .uuid(),
            uuid
        );
        assert!(
            project.catalog_input_generation().sequence()
                > before_project.catalog_input_generation().sequence()
        );
        assert!(project
            .catalog_input_generation()
            .delta_since(&before_project.catalog_input_generation())
            .is_unchanged());
        let after = scope.editor.catalog_snapshot_record();
        // The targeted transaction reruns, but equivalent catalog input intentionally retains
        // the immutable Editor generation instead of publishing a fictitious content change.
        assert!(std::sync::Arc::ptr_eq(&after, &before));
        assert_eq!(after.assets, before.assets);
        assert_eq!(after.asset(&uuid.to_string()).unwrap().locator, SOURCE);
    });
}

#[test]
fn reimport_resolves_asset_uuid_after_source_rename() {
    exercise(|scope| {
        let uuid = scope
            .snapshot()
            .asset_registry()
            .entry_by_path(&uri(SOURCE))
            .unwrap()
            .uuid();
        let renamed = uri("res://data/renamed.u09data");
        let manager = resolve_manager_service(
            &scope.runtime.handle(),
            scope.handle.as_ref().unwrap().clone(),
        )
        .unwrap();
        manager
            .relocate_project_source(uuid, renamed.clone())
            .unwrap();
        drop(manager);
        scope.editor.refresh_from_runtime_project().unwrap();
        let result = scope
            .editor
            .submit_project_asset_reimport(uuid)
            .unwrap()
            .wait_until(scope.deadline())
            .unwrap()
            .unwrap();
        assert_eq!(result.source(), &renamed);
        assert!(result.status().unwrap().imported);
        assert!(scope
            .editor
            .catalog_snapshot_record()
            .asset_by_locator(SOURCE)
            .is_none());
        assert_eq!(
            scope
                .editor
                .catalog_snapshot_record()
                .asset(&uuid.to_string())
                .unwrap()
                .locator,
            renamed.to_string()
        );
    });
}

#[test]
fn reimport_refuses_stale_source_uuid_after_old_path_is_reused() {
    use std::fs;

    exercise(|scope| {
        let original = scope.snapshot();
        let selected_uuid = original
            .asset_registry()
            .entry_by_path(&uri(SOURCE))
            .unwrap()
            .uuid();
        let (source, source_uuid) = super::reimport_source(&original, selected_uuid).unwrap();
        assert_eq!(source_uuid, selected_uuid);
        let activation = original.activation_identity();
        let renamed = uri("res://data/relocated.u09data");
        let manager = resolve_manager_service(
            &scope.runtime.handle(),
            scope.handle.as_ref().unwrap().clone(),
        )
        .unwrap();
        manager
            .relocate_project_source(source_uuid, renamed.clone())
            .unwrap();
        let replacement_bytes = b"{\"replacement\":true}";
        let replacement_path = scope.assets_root().join("data/item.u09data");
        fs::write(&replacement_path, replacement_bytes).unwrap();
        manager
            .import_asset_for_project(activation, SOURCE)
            .unwrap()
            .unwrap();
        let replacement = scope.snapshot();
        let replacement_uuid = replacement
            .asset_registry()
            .entry_by_path(&source)
            .unwrap()
            .uuid();
        assert_ne!(replacement_uuid, source_uuid);
        let replacement_meta_path = scope.assets_root().join("data/item.u09data.zmeta");
        let replacement_meta = fs::read(&replacement_meta_path).unwrap();
        let calls = scope.calls.load(Ordering::SeqCst);
        let sequence = replacement.catalog_input_generation().sequence();

        // The source moved after Editor resolved its UUID to this URI. The Runtime clone
        // fence must reject the captured identity before invoking the replacement importer.
        let error = manager
            .reimport_project_source_for_project(activation, source_uuid, &source.to_string())
            .unwrap_err();
        assert!(error.to_string().contains("source identity changed"));
        assert_eq!(scope.calls.load(Ordering::SeqCst), calls);
        assert_eq!(fs::read(&replacement_path).unwrap(), replacement_bytes);
        assert_eq!(fs::read(&replacement_meta_path).unwrap(), replacement_meta);
        let after = scope.snapshot();
        assert_eq!(after.catalog_input_generation().sequence(), sequence);
        assert_eq!(
            after
                .asset_registry()
                .entry_by_path(&source)
                .unwrap()
                .uuid(),
            replacement_uuid
        );
        assert_eq!(
            after
                .asset_registry()
                .entry_by_uuid(source_uuid)
                .unwrap()
                .path(),
            &renamed
        );
    });
}

#[test]
fn reimport_subasset_uses_root_owner_and_preserves_selected_uuid() {
    use std::fs;

    exercise(|scope| {
        fs::write(
            scope.assets_root().join("data/item.u09data"),
            b"{\"with_subasset\":true}",
        )
        .unwrap();
        let manager = resolve_manager_service(
            &scope.runtime.handle(),
            scope.handle.as_ref().unwrap().clone(),
        )
        .unwrap();
        let original = scope.snapshot();
        manager
            .import_asset_for_project(original.activation_identity(), SOURCE)
            .unwrap()
            .unwrap();
        drop(manager);
        scope.editor.refresh_from_runtime_project().unwrap();
        let project = scope.snapshot();
        let root_uuid = project
            .asset_registry()
            .entry_by_path(&uri(SOURCE))
            .unwrap()
            .uuid();
        let child = uri("res://data/item.u09data#detail");
        let child_uuid = project
            .asset_registry()
            .entry_by_path(&child)
            .unwrap()
            .uuid();
        assert_ne!(child_uuid, root_uuid);
        let (source, owner) = super::reimport_source(&project, child_uuid).unwrap();
        assert_eq!(source, uri(SOURCE));
        assert_eq!(owner, root_uuid);
        let calls = scope.calls.load(Ordering::SeqCst);
        let result = scope
            .editor
            .submit_project_asset_reimport(child_uuid)
            .unwrap()
            .wait_until(scope.deadline())
            .unwrap()
            .unwrap();
        assert_eq!(result.target_uuid(), &child_uuid);
        assert_eq!(result.source(), &source);
        assert!(result.status().unwrap().imported);
        assert_eq!(scope.calls.load(Ordering::SeqCst), calls + 1);
        assert_eq!(
            scope
                .snapshot()
                .asset_registry()
                .entry_by_path(&child)
                .unwrap()
                .uuid(),
            child_uuid
        );
    });
}

#[test]
fn queued_reimport_refuses_a_reopened_project_activation() {
    super::super::project_mutation_tests::exercise_with_reopen(|scope| {
        let uuid = scope
            .snapshot()
            .asset_registry()
            .entry_by_path(&uri(SOURCE))
            .unwrap()
            .uuid();
        let gate = scope.block_mutation_queue();
        let ticket = scope.editor.submit_project_asset_reimport(uuid).unwrap();
        scope.reopen();
        let calls = scope.calls.load(Ordering::SeqCst);
        drop(gate);
        let error = ticket.wait_until(scope.deadline()).unwrap().unwrap_err();
        assert!(error.to_string().contains("inactive"));
        assert_eq!(scope.calls.load(Ordering::SeqCst), calls);
    });
}

#[cfg(unix)]
#[test]
fn reimport_refuses_replaced_leaf_parent_and_dangling_symlinks() {
    use std::fs;
    use std::os::unix::fs::symlink;

    exercise(|scope| {
        let uuid = scope
            .snapshot()
            .asset_registry()
            .entry_by_path(&uri(SOURCE))
            .unwrap()
            .uuid();
        let assets = scope.assets_root();
        let source = assets.join("data/item.u09data");
        let original = assets.join("data/original.u09data");
        let outside = scope.root().join("outside.u09data");
        let outside_bytes = b"{\"outside\":true}";
        fs::write(&outside, outside_bytes).unwrap();
        fs::rename(&source, &original).unwrap();
        symlink(&outside, &source).unwrap();
        let leaf = scope
            .editor
            .submit_project_asset_reimport(uuid)
            .unwrap()
            .wait_until(scope.deadline())
            .unwrap();
        assert!(leaf.is_err(), "reimport must refuse a linked source leaf");
        assert_eq!(fs::read(&outside).unwrap(), outside_bytes);
        fs::remove_file(&source).unwrap();
        symlink(scope.root().join("missing-source.u09data"), &source).unwrap();
        let dangling = scope
            .editor
            .submit_project_asset_reimport(uuid)
            .unwrap()
            .wait_until(scope.deadline())
            .unwrap();
        assert!(
            dangling.is_err(),
            "reimport must refuse a dangling source link"
        );
        fs::remove_file(&source).unwrap();
        fs::rename(&original, &source).unwrap();

        let data = assets.join("data");
        let retained = assets.join("retained-data");
        fs::rename(&data, &retained).unwrap();
        symlink(&retained, &data).unwrap();
        let parent = scope
            .editor
            .submit_project_asset_reimport(uuid)
            .unwrap()
            .wait_until(scope.deadline())
            .unwrap();
        assert!(
            parent.is_err(),
            "reimport must refuse a linked source ancestor"
        );
        assert_eq!(fs::read(&outside).unwrap(), outside_bytes);
        fs::remove_file(&data).unwrap();
        fs::rename(&retained, &data).unwrap();
        assert!(scope
            .editor
            .catalog_snapshot_record()
            .asset(&uuid.to_string())
            .is_some());
    });
}

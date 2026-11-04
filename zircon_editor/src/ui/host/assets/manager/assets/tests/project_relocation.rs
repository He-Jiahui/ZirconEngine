use zircon_runtime::asset::{AssetUri, AssetUuid};
use zircon_runtime::core::CoreError;

use super::DefaultEditorAssetManager;

#[test]
fn relocation_requires_the_runtime_asset_owner() {
    let error = DefaultEditorAssetManager::new()
        .submit_project_source_relocation(
            AssetUuid::from_stable_label("editor-relocation-requires-runtime"),
            AssetUri::parse("res://relocated.asset").unwrap(),
        )
        .unwrap_err();

    assert_eq!(
        error,
        CoreError::Initialization(
            crate::ui::host::module::EDITOR_ASSET_MANAGER_NAME.to_string(),
            "project source relocation requires the runtime asset manager".to_string(),
        )
    );
}

#[test]
fn asset_rename_job_preserves_uuid_and_publishes_catalog() {
    use super::super::project_mutation_tests::{exercise, uri, SOURCE};
    exercise(|scope| {
        let uuid = scope
            .snapshot()
            .asset_registry()
            .entry_by_path(&uri(SOURCE))
            .unwrap()
            .uuid();
        let target = uri("res://data/renamed.u09data");
        let result = scope
            .editor
            .submit_project_source_relocation(uuid, target.clone())
            .unwrap()
            .wait_until(scope.deadline())
            .unwrap()
            .unwrap();
        assert_eq!(result.source_uuid(), &uuid);
        assert_eq!(result.target(), &target);
        assert!(result.changed());
        assert_eq!(
            scope
                .snapshot()
                .asset_registry()
                .entry_by_uuid(uuid)
                .unwrap()
                .path(),
            &target
        );
        assert_eq!(
            scope
                .editor
                .catalog_snapshot_record()
                .asset(&uuid.to_string())
                .unwrap()
                .locator,
            target.to_string()
        );
    });
}

#[test]
fn queued_asset_rename_refuses_a_reopened_project_activation() {
    use super::super::project_mutation_tests::{exercise_with_reopen as exercise, uri, SOURCE};
    exercise(|scope| {
        let uuid = scope
            .snapshot()
            .asset_registry()
            .entry_by_path(&uri(SOURCE))
            .unwrap()
            .uuid();
        let gate = scope.block_mutation_queue();
        let ticket = scope
            .editor
            .submit_project_source_relocation(uuid, uri("res://data/renamed.u09data"))
            .unwrap();
        scope.reopen();
        drop(gate);
        let error = ticket.wait_until(scope.deadline()).unwrap().unwrap_err();
        assert!(error.to_string().contains("inactive"));
        assert!(scope.assets_root().join("data/item.u09data").is_file());
        assert!(!scope.assets_root().join("data/renamed.u09data").exists());
    });
}

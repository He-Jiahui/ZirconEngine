use crate::core::asset::{DirtyExternalEffectId, SaveDirtyViewOutcomeStatus};
use crate::core::extension::SaveReason;
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::{DirtyDocumentSaveOwner, DirtyDocumentSaveStart, EditorError, EditorManager};

use super::support::{
    cleanup_theme_project, setup_theme_project, DUPLICATE_IMPORTED_THEME_ASSET,
    DUPLICATE_THEME_UI_LAYOUT_ASSET,
};

#[test]
fn foreground_ui_save_completes_source_dirty_effect() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let (config_path, project_root, runtime) = setup_theme_project(
        "zircon_editor_theme_save_ownership",
        "zircon_editor_theme_save_ownership_project",
        DUPLICATE_THEME_UI_LAYOUT_ASSET,
        DUPLICATE_IMPORTED_THEME_ASSET,
    );
    let manager_handle = runtime
        .resolve_manager_handle::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let manager = manager_handle.enter().unwrap();
    manager.open_project(&project_root).unwrap();
    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.zui", None)
        .unwrap();
    assert!(manager
        .apply_ui_asset_editor_theme_refactor_item(&instance_id, 0)
        .unwrap());
    let dirty = manager.dirty_document_toolkits().unwrap();
    assert_eq!(dirty.len(), 1);
    let document = dirty[0].document_id;
    let before = manager
        .context()
        .dirty_documents()
        .snapshot(document)
        .unwrap();
    assert!(before.is_dirty());
    assert!(before
        .external_effects()
        .contains(&DirtyExternalEffectId::ui_source_buffer()));
    drop(manager);

    let saved = EditorManager::save_ui_asset_editor(manager_handle.clone(), &instance_id)
        .expect("foreground UI save should complete through its owner");
    assert_eq!(
        std::fs::read_to_string(project_root.join("assets/ui/layouts/editor.zui")).unwrap(),
        saved
    );
    let manager = manager_handle.enter().unwrap();
    let after = manager
        .context()
        .dirty_documents()
        .snapshot(document)
        .unwrap();
    assert!(!after.is_dirty());
    assert!(after.external_effects().is_empty());
    assert!(after.generation() > before.generation());
    assert!(manager.dirty_document_toolkits().unwrap().is_empty());
    assert!(
        !manager
            .current_view_instances()
            .iter()
            .find(|view| view.instance_id == instance_id)
            .unwrap()
            .dirty
    );

    cleanup_theme_project(&config_path, &project_root);
}

#[test]
fn queued_ui_save_rejects_concurrent_structured_edit() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let (config_path, project_root, runtime) = setup_theme_project(
        "zircon_editor_theme_save_concurrent_edit",
        "zircon_editor_theme_save_concurrent_edit_project",
        DUPLICATE_THEME_UI_LAYOUT_ASSET,
        DUPLICATE_IMPORTED_THEME_ASSET,
    );
    let manager_handle = runtime
        .resolve_manager_handle::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let manager = manager_handle.enter().unwrap();
    manager.open_project(&project_root).unwrap();
    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.zui", None)
        .unwrap();
    assert!(manager
        .apply_ui_asset_editor_theme_refactor_item(&instance_id, 0)
        .unwrap());
    let dirty = manager.dirty_document_toolkits().unwrap();
    assert_eq!(dirty.len(), 1);
    let document = dirty[0].document_id;
    let before = manager
        .context()
        .dirty_documents()
        .snapshot(document)
        .unwrap();
    drop(manager);

    assert_eq!(
        EditorManager::begin_dirty_document_save(
            manager_handle.clone(),
            DirtyDocumentSaveOwner::ClosePrompt,
            [document],
            SaveReason::Close,
        )
        .unwrap(),
        DirtyDocumentSaveStart::Scheduled
    );
    let manager = manager_handle.enter().unwrap();
    assert!(manager
        .apply_ui_asset_editor_theme_refactor_item(&instance_id, 0)
        .expect("second structured edit must change the UI source"));
    let edited = manager
        .context()
        .dirty_documents()
        .snapshot(document)
        .unwrap();
    assert!(edited.generation() > before.generation());

    // Completion applies the admission snapshot even if the hook writes edit 2.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    let result = loop {
        if let Some(result) = manager
            .poll_dirty_document_save(DirtyDocumentSaveOwner::ClosePrompt)
            .unwrap()
        {
            break result;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "save did not terminalize"
        );
        std::thread::yield_now();
    };
    assert_eq!(result.outcomes().len(), 1);
    let outcome = &result.outcomes()[0];
    assert_eq!(outcome.document_id(), document);
    assert_eq!(
        outcome.status(),
        &SaveDirtyViewOutcomeStatus::StaleGeneration
    );
    let report = outcome
        .save_report()
        .expect("real UI hook must write source");
    assert!(report.written_bytes() > 0);
    assert!(report.cooperating_source_writes_are_serialized());
    assert_eq!(manager.dirty_document_toolkits().unwrap().len(), 1);
    let after = manager
        .context()
        .dirty_documents()
        .snapshot(document)
        .unwrap();
    assert!(after.is_dirty());
    assert!(after
        .external_effects()
        .contains(&DirtyExternalEffectId::ui_source_buffer()));
    assert!(
        manager
            .current_view_instances()
            .iter()
            .find(|view| view.instance_id == instance_id)
            .unwrap()
            .dirty
    );
    assert!(matches!(
        manager.close_view(&instance_id),
        Err(EditorError::DocumentCloseNeedsDecision { .. })
    ));

    cleanup_theme_project(&config_path, &project_root);
}

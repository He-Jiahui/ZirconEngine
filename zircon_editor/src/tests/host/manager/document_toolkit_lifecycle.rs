use std::fs;

use crate::core::asset::{DirtyExternalEffectId, SaveDirtyViewOutcomeStatus};
use crate::core::extension::SaveReason;
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::{DirtyDocumentSaveOwner, DirtyDocumentSaveStart, EditorError, EditorManager};
use crate::ui::workbench::layout::{LayoutCommand, MainPageId};
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};
use zircon_runtime::core::manager::ManagerResolver;

use super::support::*;

#[test]
fn project_ui_asset_document_persists_the_canonical_toolkit_route() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_document_toolkit_canonical_route");
    let project_root = unique_temp_dir("zircon_editor_document_toolkit_canonical_route_project");
    let ui_asset_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.zui");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);
    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.zui", None)
        .expect("project UI asset editor should open");
    let workspace = manager.project_workspace();
    manager
        .apply_project_workspace(Some(workspace))
        .expect("canonical UI asset toolkit route should restore");
    manager
        .ui_asset_editor_reflection(&instance_id)
        .expect("restored UI asset editor should remain available");
    assert!(manager
        .document_toolkit_snapshot()
        .descriptors()
        .iter()
        .any(|descriptor| descriptor.instance_id().as_str() == instance_id.0));
    let instance = manager
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.instance_id == instance_id)
        .expect("opened UI asset editor should be persisted in the workspace");

    assert_eq!(
        instance
            .serializable_payload
            .get("asset_locator")
            .and_then(serde_json::Value::as_str),
        Some("res://ui/layouts/editor.zui")
    );
    assert_eq!(
        instance
            .serializable_payload
            .get("open_operation")
            .and_then(serde_json::Value::as_str),
        Some("view.editor.ui_asset.open")
    );
    assert!(
        instance.serializable_payload.get("asset_id").is_none(),
        "workspace persistence must not retain the replaced UI-editor route"
    );
    assert!(manager
        .apply_layout_command(LayoutCommand::CloseView {
            instance_id: instance_id.clone(),
        })
        .expect("restored UI asset editor should close"));
    assert!(!manager
        .document_toolkit_snapshot()
        .descriptors()
        .iter()
        .any(|descriptor| descriptor.instance_id().as_str() == instance_id.0));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn project_ui_asset_opened_by_absolute_path_persists_a_canonical_toolkit_route() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_document_toolkit_absolute_route");
    let project_root = unique_temp_dir("zircon_editor_document_toolkit_absolute_route_project");
    let ui_asset_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.zui");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);
    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("project UI asset should open from its absolute source path");
    assert_eq!(
        manager
            .ui_asset_editor_reflection(&instance_id)
            .expect("absolute project path should produce an editor session")
            .route
            .asset_id,
        "res://ui/layouts/editor.zui"
    );
    let workspace = manager.project_workspace();
    manager
        .apply_project_workspace(Some(workspace))
        .expect("absolute project path should restore through its canonical toolkit route");
    let instance = manager
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.instance_id == instance_id)
        .expect("opened UI asset editor should be persisted in the workspace");

    assert_eq!(
        instance
            .serializable_payload
            .get("asset_locator")
            .and_then(serde_json::Value::as_str),
        Some("res://ui/layouts/editor.zui")
    );
    assert_eq!(
        instance
            .serializable_payload
            .get("open_operation")
            .and_then(serde_json::Value::as_str),
        Some("view.editor.ui_asset.open")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn project_workspace_rejects_ui_asset_source_outside_asset_roots() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_document_toolkit_reject_external_source");
    let project_root =
        unique_temp_dir("zircon_editor_document_toolkit_reject_external_source_project");
    let external_asset_path =
        unique_temp_dir("zircon_editor_document_toolkit_reject_external_source_file")
            .join("external.zui");
    fs::create_dir_all(external_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&external_asset_path, STYLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);
    manager.open_project(&project_root).unwrap();

    let error = manager
        .open_ui_asset_editor(&external_asset_path, None)
        .expect_err("project workspaces must reject UI asset sources outside project roots");
    assert!(error
        .to_string()
        .contains("outside the active project asset roots"));
    assert!(!manager
        .current_view_instances()
        .iter()
        .any(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.ui_asset")));
    assert!(manager.document_toolkit_snapshot().descriptors().is_empty());

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
    let _ = fs::remove_dir_all(external_asset_path.parent().unwrap());
}

#[test]
fn workspace_restore_rejects_the_replaced_ui_asset_route() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_document_toolkit_reject_legacy_route");
    let project_root =
        unique_temp_dir("zircon_editor_document_toolkit_reject_legacy_route_project");
    let ui_asset_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.zui");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);
    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.zui", None)
        .expect("project UI asset editor should open");
    let mut workspace = manager.project_workspace();
    let instance = workspace
        .open_view_instances
        .iter_mut()
        .find(|instance| instance.instance_id == instance_id)
        .expect("opened UI asset editor should be persisted in the workspace");
    instance.serializable_payload = serde_json::json!({
        "asset_id": "res://ui/layouts/editor.zui",
        "asset_kind": "layout",
        "mode": "Design",
        "preview_preset": "EditorDocked",
    });

    let error = manager
        .apply_project_workspace(Some(workspace))
        .expect_err("the replaced UI-editor route must not restore");
    assert!(error.to_string().contains("invalid asset toolkit route"));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn layout_close_command_unregisters_the_document_toolkit() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_layout_close_document_toolkit");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_layout_close_document_toolkit_file").join("style.zui");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");

    assert!(manager
        .document_toolkit_snapshot()
        .descriptors()
        .iter()
        .any(|descriptor| descriptor.instance_id().as_str() == instance_id.0));

    assert!(manager
        .apply_layout_command(LayoutCommand::CloseView {
            instance_id: instance_id.clone(),
        })
        .expect("layout close should succeed"));

    assert!(!manager
        .document_toolkit_snapshot()
        .descriptors()
        .iter()
        .any(|descriptor| descriptor.instance_id().as_str() == instance_id.0));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn layout_close_command_keeps_non_document_instance_ids_as_no_ops() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_layout_close_non_document");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    for instance_id in [
        ViewInstanceId::new(String::new()),
        ViewInstanceId::new("x".repeat(257)),
    ] {
        assert!(!manager
            .apply_layout_command(LayoutCommand::CloseView { instance_id })
            .expect("unknown non-document instance should retain close no-op semantics"));
    }

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn dirty_close_refuses_without_removing_layout_session_or_toolkit_and_stale_discard_retries() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_dirty_close_guard");
    let ui_asset_path = unique_temp_dir("zircon_editor_dirty_close_guard_file").join("test.zui");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);
    let runtime = editor_runtime_with_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance_id = manager.open_ui_asset_editor(&ui_asset_path, None).unwrap();
    manager
        .mark_document_external_effect(&instance_id, DirtyExternalEffectId::ui_source_buffer())
        .unwrap();
    let dirty = manager.dirty_document_toolkits().unwrap().remove(0);
    let layout_before = manager.current_layout();
    let instances_before = manager.current_view_instances();
    let toolkit_before = manager.document_toolkit_snapshot();

    for close in [
        manager.close_view(&instance_id),
        manager.apply_layout_command(LayoutCommand::CloseView {
            instance_id: instance_id.clone(),
        }),
        manager.apply_layout_command(LayoutCommand::ResetToDefault),
    ] {
        assert!(matches!(
            close,
            Err(EditorError::DocumentCloseNeedsDecision {
                document,
                dirty_generation,
            }) if document == dirty.document_id && dirty_generation == dirty.dirty_generation
        ));
        assert_eq!(manager.current_layout(), layout_before);
        assert_eq!(manager.current_view_instances(), instances_before);
        assert_eq!(
            manager.document_toolkit_snapshot().generation(),
            toolkit_before.generation()
        );
        assert_eq!(
            manager.document_toolkit_snapshot().descriptors(),
            toolkit_before.descriptors()
        );
        assert_eq!(manager.dirty_document_toolkits().unwrap().len(), 1);
    }

    manager
        .mark_document_external_effect(&instance_id, DirtyExternalEffectId::ui_source_buffer())
        .unwrap();
    assert!(matches!(
        manager.close_view_discarding(&instance_id, dirty.document_id, dirty.close_revision),
        Err(EditorError::DocumentCloseDecisionStale { document, .. })
            if document == dirty.document_id
    ));
    assert_eq!(manager.current_layout(), layout_before);
    assert_eq!(manager.current_view_instances(), instances_before);
    assert_eq!(
        manager.document_toolkit_snapshot().generation(),
        toolkit_before.generation()
    );
    assert_eq!(
        manager.document_toolkit_snapshot().descriptors(),
        toolkit_before.descriptors()
    );

    let current = manager.dirty_document_toolkits().unwrap().remove(0);
    assert!(manager
        .close_view_discarding(&instance_id, current.document_id, current.close_revision)
        .unwrap());
    assert!(manager.dirty_document_toolkits().unwrap().is_empty());
    assert!(!manager
        .current_view_instances()
        .iter()
        .any(|instance| instance.instance_id == instance_id));
    assert!(manager.document_toolkit_snapshot().descriptors().is_empty());

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(config_path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn queued_document_save_owner_blocks_explicit_discard_until_completion_is_polled() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_close_queued_save");
    let project_root = unique_temp_dir("zircon_editor_close_queued_save_project");
    let ui_asset_path = project_root.join("assets/ui/layouts/editor.zui");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);
    let runtime = editor_runtime_with_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);
    manager.open_project(&project_root).unwrap();
    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.zui", None)
        .unwrap();
    manager
        .mark_document_external_effect(&instance_id, DirtyExternalEffectId::ui_source_buffer())
        .unwrap();
    let dirty = manager.dirty_document_toolkits().unwrap().remove(0);
    assert_eq!(
        manager
            .begin_dirty_document_save(
                DirtyDocumentSaveOwner::ClosePrompt,
                [dirty.document_id],
                SaveReason::Close,
            )
            .unwrap(),
        DirtyDocumentSaveStart::Scheduled
    );
    let layout_before = manager.current_layout();
    let instances_before = manager.current_view_instances();
    let toolkit_before = manager.document_toolkit_snapshot();
    assert!(matches!(
        manager.close_view_discarding(&instance_id, dirty.document_id, dirty.close_revision),
        Err(EditorError::DocumentCloseSaveInProgress)
    ));
    assert_eq!(manager.current_layout(), layout_before);
    assert_eq!(manager.current_view_instances(), instances_before);
    assert_eq!(
        manager.document_toolkit_snapshot().generation(),
        toolkit_before.generation()
    );
    assert_eq!(
        manager.document_toolkit_snapshot().descriptors(),
        toolkit_before.descriptors()
    );
    assert_eq!(
        manager.dirty_document_save_owner(),
        Some(DirtyDocumentSaveOwner::ClosePrompt)
    );

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        if manager
            .poll_dirty_document_save(DirtyDocumentSaveOwner::ClosePrompt)
            .unwrap()
            .is_some()
        {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "save did not terminalize"
        );
        std::thread::yield_now();
    }
    assert!(manager
        .close_view_discarding(&instance_id, dirty.document_id, dirty.close_revision)
        .unwrap());

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(config_path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn source_edit_before_queued_save_completion_stays_dirty_in_registry_and_tab() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_queued_save_source_revision");
    let project_root = unique_temp_dir("zircon_editor_queued_save_source_revision_project");
    let ui_asset_path = project_root.join("assets/ui/layouts/editor.zui");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);
    let runtime = editor_runtime_with_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);
    manager.open_project(&project_root).unwrap();
    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.zui", None)
        .unwrap();
    manager
        .update_ui_asset_editor_source(
            &instance_id,
            STYLE_UI_LAYOUT_ASSET.replace("Styled UI Asset", "First Source"),
        )
        .unwrap();
    let first = manager.dirty_document_toolkits().unwrap().remove(0);
    assert_eq!(
        manager
            .begin_dirty_document_save(
                DirtyDocumentSaveOwner::ClosePrompt,
                [first.document_id],
                SaveReason::Close,
            )
            .unwrap(),
        DirtyDocumentSaveStart::Scheduled
    );
    manager
        .update_ui_asset_editor_source(
            &instance_id,
            STYLE_UI_LAYOUT_ASSET.replace("Styled UI Asset", "Second Source"),
        )
        .unwrap();
    let second = manager.dirty_document_toolkits().unwrap().remove(0);
    assert!(second.dirty_generation > first.dirty_generation);

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
    assert!(result
        .outcomes()
        .iter()
        .all(|outcome| !matches!(outcome.status(), SaveDirtyViewOutcomeStatus::Saved { .. })));
    assert_eq!(manager.dirty_document_toolkits().unwrap().len(), 1);
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

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(config_path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn failed_document_save_keeps_dirty_view_until_successful_retry() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_close_failed_save");
    let project_root = unique_temp_dir("zircon_editor_close_failed_save_project");
    let ui_asset_path = project_root.join("assets/ui/layouts/editor.zui");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);
    let runtime = editor_runtime_with_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);
    manager.open_project(&project_root).unwrap();
    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.zui", None)
        .unwrap();
    manager
        .update_ui_asset_editor_source(
            &instance_id,
            STYLE_UI_LAYOUT_ASSET.replace("Styled UI Asset", "Local UI Asset"),
        )
        .unwrap();
    fs::write(
        &ui_asset_path,
        STYLE_UI_LAYOUT_ASSET.replace("Styled UI Asset", "External UI Asset"),
    )
    .unwrap();
    assert!(manager.save_ui_asset_editor(&instance_id).is_err());
    assert!(matches!(
        manager.close_view(&instance_id),
        Err(EditorError::DocumentCloseNeedsDecision { .. })
    ));
    assert!(manager
        .current_view_instances()
        .iter()
        .any(|instance| instance.instance_id == instance_id));
    assert_eq!(manager.dirty_document_toolkits().unwrap().len(), 1);
    fs::write(&ui_asset_path, STYLE_UI_LAYOUT_ASSET).unwrap();
    manager.save_ui_asset_editor(&instance_id).unwrap();
    assert!(manager.close_view(&instance_id).unwrap());
    assert!(manager.document_toolkit_snapshot().descriptors().is_empty());

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(config_path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn public_batch_close_cannot_forge_a_discard_decision() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_public_batch_close_forgery");
    let ui_asset_path = unique_temp_path("zircon_editor_public_batch_close_forgery_asset")
        .with_extension("ui.toml");
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);
    let runtime = editor_runtime_with_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance_id = manager.open_ui_asset_editor(&ui_asset_path, None).unwrap();
    let window_id = MainPageId::new("window:public-forgery");
    manager
        .apply_layout_command(LayoutCommand::DetachViewToWindow {
            instance_id: instance_id.clone(),
            new_window: window_id.clone(),
        })
        .unwrap();
    manager
        .mark_document_external_effect(&instance_id, DirtyExternalEffectId::ui_source_buffer())
        .unwrap();
    let dirty = manager.dirty_document_toolkits().unwrap().remove(0);
    let before = manager.current_layout();
    let command = LayoutCommand::CloseViews {
        window_id,
        instance_ids: vec![instance_id.clone()],
    };
    let mut wire = serde_json::to_value(command).unwrap();
    wire["CloseViews"]["discard"] =
        serde_json::json!([(instance_id.clone(), dirty.document_id, dirty.close_revision)]);
    if let Ok(forged) = serde_json::from_value::<LayoutCommand>(wire) {
        assert!(matches!(
            manager.apply_layout_command(forged),
            Err(EditorError::DocumentCloseNeedsDecision { .. })
        ));
    }
    assert_eq!(manager.current_layout(), before);
    assert_eq!(manager.dirty_document_toolkits().unwrap().len(), 1);
    assert!(manager
        .current_view_instances()
        .iter()
        .any(|view| view.instance_id == instance_id));
    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(config_path);
    let _ = fs::remove_file(ui_asset_path);
}

#[test]
fn rejected_wrong_kind_source_edit_does_not_leave_unregistered_dirty_buffer() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_wrong_kind_source_edit");
    let ui_asset_path =
        unique_temp_path("zircon_editor_wrong_kind_source_edit_asset").with_extension("ui.toml");
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);
    let runtime = editor_runtime_with_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance_id = manager.open_ui_asset_editor(&ui_asset_path, None).unwrap();
    let wrong_kind = r#"
[asset]
kind = "widget"
id = "ui.widgets.button"
version = 1
display_name = "Toolbar Button"

[root]
node = "button_root"

[components.ToolbarButton]
root = "button_root"

[nodes.button_root]
kind = "native"
type = "Button"
control_id = "ToolbarButton"
props = { text = "Press" }
"#;
    assert!(manager
        .update_ui_asset_editor_source(&instance_id, wrong_kind)
        .is_err());
    assert!(manager.dirty_document_toolkits().unwrap().is_empty());
    assert!(manager.close_view(&instance_id).unwrap());
    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(config_path);
    let _ = fs::remove_file(ui_asset_path);
}

#[test]
fn each_ui_source_edit_advances_the_dirty_revision_used_by_save_and_close() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_source_dirty_revision");
    let ui_asset_path =
        unique_temp_path("zircon_editor_source_dirty_revision_asset").with_extension("ui.toml");
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);
    let runtime = editor_runtime_with_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance_id = manager.open_ui_asset_editor(&ui_asset_path, None).unwrap();
    manager
        .update_ui_asset_editor_source(
            &instance_id,
            STYLE_UI_LAYOUT_ASSET.replace("Styled UI Asset", "First Source"),
        )
        .unwrap();
    let first = manager.dirty_document_toolkits().unwrap().remove(0);
    manager
        .update_ui_asset_editor_source(
            &instance_id,
            STYLE_UI_LAYOUT_ASSET.replace("Styled UI Asset", "Second Source"),
        )
        .unwrap();
    let second = manager.dirty_document_toolkits().unwrap().remove(0);
    assert_eq!(second.document_id, first.document_id);
    assert!(second.dirty_generation > first.dirty_generation);
    assert_ne!(second.close_revision, first.close_revision);
    assert!(matches!(
        manager.close_view_discarding(&instance_id, first.document_id, first.close_revision),
        Err(EditorError::DocumentCloseDecisionStale { .. })
    ));
    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(config_path);
    let _ = fs::remove_file(ui_asset_path);
}

#[test]
fn structured_edit_after_session_reload_advances_external_dirty_generation() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_reload_source_revision");
    let ui_asset_path =
        unique_temp_path("zircon_editor_reload_source_revision_asset").with_extension("ui.toml");
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);
    let runtime = editor_runtime_with_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance_id = manager.open_ui_asset_editor(&ui_asset_path, None).unwrap();
    manager
        .update_ui_asset_editor_source(
            &instance_id,
            STYLE_UI_LAYOUT_ASSET.replace("Styled UI Asset", "First Source"),
        )
        .unwrap();
    assert!(manager
        .reload_ui_asset_editor_from_disk(&instance_id)
        .unwrap());
    let before = manager.dirty_document_toolkits().unwrap().remove(0);
    assert!(manager
        .upsert_ui_asset_editor_style_token(&instance_id, "surface_fill", "#223344")
        .unwrap());
    let after = manager.dirty_document_toolkits().unwrap().remove(0);
    assert!(after.dirty_generation > before.dirty_generation);
    assert_ne!(after.close_revision, before.close_revision);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(config_path);
    let _ = fs::remove_file(ui_asset_path);
}

#[test]
fn floating_batch_discard_refuses_a_stale_second_tab_before_removing_either() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_batch_stale_second");
    let first_path = unique_temp_path("zircon_editor_batch_stale_first").with_extension("ui.toml");
    let second_path =
        unique_temp_path("zircon_editor_batch_stale_second_asset").with_extension("ui.toml");
    write_ui_asset(&first_path, STYLE_UI_LAYOUT_ASSET);
    write_ui_asset(&second_path, STYLE_UI_LAYOUT_ASSET);
    let runtime = editor_runtime_with_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let first = manager.open_ui_asset_editor(&first_path, None).unwrap();
    let second = manager.open_ui_asset_editor(&second_path, None).unwrap();
    let window_id = MainPageId::new("window:stale-second");
    for instance_id in [&first, &second] {
        manager
            .apply_layout_command(LayoutCommand::DetachViewToWindow {
                instance_id: instance_id.clone(),
                new_window: window_id.clone(),
            })
            .unwrap();
        manager
            .mark_document_external_effect(instance_id, DirtyExternalEffectId::ui_source_buffer())
            .unwrap();
    }
    let dirty = manager.dirty_document_toolkits().unwrap();
    let first_decision = dirty.iter().find(|view| view.instance_id == first).unwrap();
    let second_decision = dirty
        .iter()
        .find(|view| view.instance_id == second)
        .unwrap();
    let decisions = vec![
        (
            first.clone(),
            first_decision.document_id,
            first_decision.close_revision,
        ),
        (
            second.clone(),
            second_decision.document_id,
            second_decision.close_revision,
        ),
    ];
    manager
        .update_ui_asset_editor_source(
            &second,
            STYLE_UI_LAYOUT_ASSET.replace("Styled UI Asset", "Second Tab Changed"),
        )
        .unwrap();
    let layout_before = manager.current_layout();
    let toolkits_before = manager.document_toolkit_snapshot();
    assert!(matches!(
        manager.close_views_with_discard(&window_id, &[first.clone(), second.clone()], &decisions),
        Err(EditorError::DocumentCloseDecisionStale { .. })
    ));
    assert_eq!(manager.current_layout(), layout_before);
    assert_eq!(
        manager.document_toolkit_snapshot().descriptors(),
        toolkits_before.descriptors()
    );
    assert!(manager
        .current_view_instances()
        .iter()
        .any(|view| view.instance_id == first));
    assert!(manager
        .current_view_instances()
        .iter()
        .any(|view| view.instance_id == second));
    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(config_path);
    let _ = fs::remove_file(first_path);
    let _ = fs::remove_file(second_path);
}

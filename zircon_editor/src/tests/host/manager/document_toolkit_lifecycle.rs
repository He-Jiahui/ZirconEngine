use std::fs;

use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::workbench::layout::LayoutCommand;
use crate::ui::workbench::view::ViewInstanceId;
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
        .any(|instance| instance.descriptor_id.as_str() == "editor.ui_asset"));
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

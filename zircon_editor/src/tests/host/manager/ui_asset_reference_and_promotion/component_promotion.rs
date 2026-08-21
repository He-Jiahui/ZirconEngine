use super::*;

#[test]
fn editor_manager_promotes_selected_ui_asset_component_to_external_widget_asset() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_promote_widget");
    let project_root = unique_temp_dir("zircon_editor_asset_promote_widget_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);

    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.zui");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    write_ui_asset(&layout_path, STYLE_UI_LAYOUT_ASSET);

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.zui", None)
        .expect("ui asset editor should open from project asset id");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select button");
    assert!(manager
        .extract_ui_asset_editor_selected_node_to_component(&instance_id)
        .expect("extract selected node to local component"));
    assert!(
        manager
            .ui_asset_editor_pane_presentation(&instance_id)
            .expect("pane before promote")
            .can_promote_to_external_widget
    );

    assert!(manager
        .promote_ui_asset_editor_selected_component_to_external_widget(&instance_id)
        .expect("promote selected component to external widget"));

    let promoted = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after promote");
    assert!(promoted.can_open_reference);
    assert!(!promoted.can_promote_to_external_widget);
    assert!(promoted
        .palette_items
        .iter()
        .any(|item| item == "Reference / SaveButton"));

    let widget_path = project_root
        .join("assets")
        .join("ui")
        .join("widgets")
        .join("save_button.zui");
    let widget_source = fs::read_to_string(&widget_path).expect("promoted widget file");
    let widget_asset = UiZuiAssetLoader::load_zui_str(&widget_source).expect("widget asset");
    assert_eq!(widget_asset.asset.id, "ui.widgets.save_button");
    assert_eq!(widget_asset.asset.kind, UiV2AssetKind::Component);
    assert_eq!(widget_asset.asset.version, UI_V2_ASSET_SCHEMA_VERSION);
    assert!(widget_asset.root.is_none());
    assert_eq!(
        widget_asset.components["SaveButton"].root,
        "savebutton_root"
    );

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    let button = document.node("button").expect("button node");
    assert_eq!(button.kind, UiNodeDefinitionKind::Reference);
    assert_eq!(
        button.component_ref.as_deref(),
        Some("res://ui/widgets/save_button.zui#SaveButton")
    );
    assert!(document
        .imports
        .widgets
        .iter()
        .any(|reference| { reference == "res://ui/widgets/save_button.zui#SaveButton" }));
    assert!(!document.components.contains_key("SaveButton"));

    assert!(manager
        .undo_ui_asset_editor(&instance_id)
        .expect("undo promote selected component"));
    assert!(!widget_path.exists());
    let undone = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after undo promote");
    assert!(!undone.can_open_reference);
    assert!(undone.can_promote_to_external_widget);
    assert!(!undone
        .palette_items
        .iter()
        .any(|item| item == "Reference / SaveButton"));

    assert!(manager
        .redo_ui_asset_editor(&instance_id)
        .expect("redo promote selected component"));
    assert!(widget_path.exists());
    let redone = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after redo promote");
    assert!(redone.can_open_reference);
    assert!(!redone.can_promote_to_external_widget);
    let redone_widget_source = fs::read_to_string(&widget_path).expect("redone widget file");
    let redone_widget =
        UiZuiAssetLoader::load_zui_str(&redone_widget_source).expect("redone widget asset");
    assert_eq!(redone_widget.asset.id, "ui.widgets.save_button");

    let opened = manager
        .open_ui_asset_editor_selected_reference(&instance_id)
        .expect("open promoted reference")
        .expect("reference editor instance");
    let reflection = manager
        .ui_asset_editor_reflection(&opened)
        .expect("promoted widget reflection");
    assert_eq!(
        reflection.route.asset_id,
        "res://ui/widgets/save_button.zui"
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_uses_custom_promote_widget_draft_values() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_promote_widget_custom");
    let project_root = unique_temp_dir("zircon_editor_asset_promote_widget_custom_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);

    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.zui");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    write_ui_asset(&layout_path, STYLE_UI_LAYOUT_ASSET);

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.zui", None)
        .expect("ui asset editor should open from project asset id");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select button");
    assert!(manager
        .extract_ui_asset_editor_selected_node_to_component(&instance_id)
        .expect("extract selected node to local component"));
    manager
        .set_ui_asset_editor_selected_promote_widget_asset_id(
            &instance_id,
            "res://ui/widgets/custom/editor_save.zui",
        )
        .expect("set promote asset id");
    manager
        .set_ui_asset_editor_selected_promote_widget_component_name(
            &instance_id,
            "EditorSaveButton",
        )
        .expect("set promote component name");
    manager
        .set_ui_asset_editor_selected_promote_widget_document_id(
            &instance_id,
            "ui.widgets.custom.editor_save",
        )
        .expect("set promote document id");

    assert!(manager
        .promote_ui_asset_editor_selected_component_to_external_widget(&instance_id)
        .expect("promote selected component to custom external widget"));

    let widget_path = project_root
        .join("assets")
        .join("ui")
        .join("widgets")
        .join("custom")
        .join("editor_save.zui");
    let widget_source = fs::read_to_string(&widget_path).expect("custom promoted widget file");
    let widget_asset = UiZuiAssetLoader::load_zui_str(&widget_source).expect("widget asset");
    assert_eq!(widget_asset.asset.id, "ui.widgets.custom.editor_save");
    assert!(widget_asset.components.contains_key("EditorSaveButton"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    let button = document.node("button").expect("button node");
    assert_eq!(
        button.component_ref.as_deref(),
        Some("res://ui/widgets/custom/editor_save.zui#EditorSaveButton")
    );
    assert!(document.imports.widgets.iter().any(|reference| {
        reference == "res://ui/widgets/custom/editor_save.zui#EditorSaveButton"
    }));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

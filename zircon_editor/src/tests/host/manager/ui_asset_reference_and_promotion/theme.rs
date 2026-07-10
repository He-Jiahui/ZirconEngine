use super::*;

#[test]
fn editor_manager_promotes_local_theme_to_external_style_asset_and_opens_selected_theme_source() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_promote_theme");
    let project_root = unique_temp_dir("zircon_editor_asset_promote_theme_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();

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
    let before = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane before promote theme");
    assert_eq!(before.theme_selected_source_kind, "Local");
    assert!(before.theme_can_promote_local);

    assert!(manager
        .promote_ui_asset_editor_local_theme_to_external_style_asset(&instance_id)
        .expect("promote local theme to external style asset"));

    let theme_path = project_root
        .join("assets")
        .join("ui")
        .join("themes")
        .join("editor_theme.zui");
    let theme_source = fs::read_to_string(&theme_path).expect("promoted theme file");
    let theme_asset = UiZuiAssetLoader::load_zui_str(&theme_source).expect("style asset");
    assert_eq!(theme_asset.asset.id, "ui.theme.editor_theme");
    assert_eq!(theme_asset.asset.kind, UiV2AssetKind::Style);
    assert_eq!(theme_asset.asset.version, UI_V2_ASSET_SCHEMA_VERSION);
    assert_eq!(theme_asset.asset.display_name, "Styled UI Asset Theme");
    assert_eq!(
        theme_asset
            .tokens
            .get("accent")
            .and_then(toml::Value::as_str),
        Some("#4488ff")
    );

    let promoted = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after promote theme");
    assert_eq!(promoted.theme_selected_source_kind, "Imported");
    assert_eq!(
        promoted.theme_selected_source_reference,
        "res://ui/themes/editor_theme.zui"
    );
    assert!(promoted.theme_selected_source_available);
    assert!(!promoted.theme_can_promote_local);

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor after theme promote");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    assert!(document.tokens.is_empty());
    assert!(document.stylesheets.is_empty());
    assert_eq!(
        document.imports.styles,
        vec!["res://ui/themes/editor_theme.zui".to_string()]
    );

    let opened = manager
        .open_ui_asset_editor_selected_theme_source(&instance_id)
        .expect("open selected theme source")
        .expect("theme source editor instance");
    let reflection = manager
        .ui_asset_editor_reflection(&opened)
        .expect("theme source reflection");
    assert_eq!(
        reflection.route.asset_id,
        "res://ui/themes/editor_theme.zui"
    );
    assert_eq!(reflection.route.asset_kind, UiAssetKind::Style);

    assert!(manager
        .undo_ui_asset_editor(&instance_id)
        .expect("undo promote local theme"));
    assert!(!theme_path.exists());
    let undone = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after undo theme promote");
    assert_eq!(undone.theme_selected_source_kind, "Local");
    assert!(undone.theme_can_promote_local);

    assert!(manager
        .redo_ui_asset_editor(&instance_id)
        .expect("redo promote local theme"));
    assert!(theme_path.exists());
    let redone = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after redo theme promote");
    assert_eq!(redone.theme_selected_source_kind, "Imported");
    assert_eq!(
        redone.theme_selected_source_reference,
        "res://ui/themes/editor_theme.zui"
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_uses_custom_promote_theme_draft_values() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_promote_theme_custom");
    let project_root = unique_temp_dir("zircon_editor_asset_promote_theme_custom_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();

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
        .set_ui_asset_editor_promote_theme_asset_id(
            &instance_id,
            "res://ui/themes/custom/editor_shell.zui",
        )
        .expect("set promote theme asset id");
    manager
        .set_ui_asset_editor_promote_theme_document_id(&instance_id, "ui.theme.custom.editor_shell")
        .expect("set promote theme document id");
    manager
        .set_ui_asset_editor_promote_theme_display_name(&instance_id, "Editor Shell Theme")
        .expect("set promote theme display name");

    assert!(manager
        .promote_ui_asset_editor_local_theme_to_external_style_asset(&instance_id)
        .expect("promote local theme to custom external style asset"));

    let theme_path = project_root
        .join("assets")
        .join("ui")
        .join("themes")
        .join("custom")
        .join("editor_shell.zui");
    let theme_source = fs::read_to_string(&theme_path).expect("custom promoted theme file");
    let theme_asset = UiZuiAssetLoader::load_zui_str(&theme_source).expect("custom style asset");
    assert_eq!(theme_asset.asset.id, "ui.theme.custom.editor_shell");
    assert_eq!(theme_asset.asset.display_name, "Editor Shell Theme");

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor after custom theme promote");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    assert_eq!(
        document.imports.styles,
        vec!["res://ui/themes/custom/editor_shell.zui".to_string()]
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_detaches_selected_imported_theme_into_local_theme_layer() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_detach_theme");
    let project_root = unique_temp_dir("zircon_editor_asset_detach_theme_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();

    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.zui");
    let imported_theme_path = project_root
        .join("assets")
        .join("ui")
        .join("theme")
        .join("shared_theme.zui");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    fs::create_dir_all(imported_theme_path.parent().unwrap()).unwrap();
    write_ui_asset(&layout_path, DETACH_THEME_UI_LAYOUT_ASSET);
    write_ui_asset(&imported_theme_path, IMPORTED_THEME_COLLISION_ASSET);

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.zui", None)
        .expect("ui asset editor should open from project asset id");
    manager
        .select_ui_asset_editor_theme_source(&instance_id, 1)
        .expect("select imported theme");

    let before = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane before detach");
    assert_eq!(before.theme_selected_source_kind, "Imported");
    assert_eq!(
        before.theme_selected_source_reference,
        "res://ui/theme/shared_theme.zui"
    );

    assert!(manager
        .detach_ui_asset_editor_selected_theme_source_to_local(&instance_id)
        .expect("detach selected imported theme into local layer"));

    let detached = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after detach");
    assert_eq!(detached.theme_selected_source_kind, "Local");
    assert_eq!(detached.theme_selected_source_reference, "local");
    assert_eq!(
        detached.theme_selected_source_token_items,
        vec![
            "accent = \"#4488ff\"".to_string(),
            "panel = \"$shared_theme_accent\"".to_string(),
            "shared_theme_accent = \"#223344\"".to_string(),
        ]
    );
    assert_eq!(
        detached.theme_selected_source_rule_items,
        vec![
            "shared_theme_local_theme • Button".to_string(),
            "local_theme • #SaveButton".to_string(),
        ]
    );

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save detached theme ui asset");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved detached ui asset");
    assert!(document.imports.styles.is_empty());
    assert_eq!(
        document.tokens.get("accent").and_then(toml::Value::as_str),
        Some("#4488ff")
    );
    assert_eq!(
        document
            .tokens
            .get("shared_theme_accent")
            .and_then(toml::Value::as_str),
        Some("#223344")
    );
    assert_eq!(
        document.tokens.get("panel").and_then(toml::Value::as_str),
        Some("$shared_theme_accent")
    );
    assert_eq!(
        document
            .stylesheets
            .iter()
            .map(|sheet| sheet.id.as_str())
            .collect::<Vec<_>>(),
        vec!["shared_theme_local_theme", "local_theme"]
    );

    assert!(manager
        .undo_ui_asset_editor(&instance_id)
        .expect("undo detach imported theme"));
    let undone = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after undo detach");
    assert_eq!(undone.theme_selected_source_kind, "Imported");
    assert_eq!(
        undone.theme_selected_source_reference,
        "res://ui/theme/shared_theme.zui"
    );

    assert!(manager
        .redo_ui_asset_editor(&instance_id)
        .expect("redo detach imported theme"));
    let redone = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after redo detach");
    assert_eq!(redone.theme_selected_source_kind, "Local");
    assert_eq!(
        redone.theme_selected_source_token_items,
        vec![
            "accent = \"#4488ff\"".to_string(),
            "panel = \"$shared_theme_accent\"".to_string(),
            "shared_theme_accent = \"#223344\"".to_string(),
        ]
    );
    let imported_theme_source =
        fs::read_to_string(&imported_theme_path).expect("imported theme source should remain");
    let imported_theme =
        UiZuiAssetLoader::load_zui_str(&imported_theme_source).expect("imported theme asset");
    assert_eq!(imported_theme.asset.id, "ui.theme.shared_theme");

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_clones_selected_imported_theme_into_local_theme_layer() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_clone_theme");
    let project_root = unique_temp_dir("zircon_editor_asset_clone_theme_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();

    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.zui");
    let imported_theme_path = project_root
        .join("assets")
        .join("ui")
        .join("theme")
        .join("shared_theme.zui");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    fs::create_dir_all(imported_theme_path.parent().unwrap()).unwrap();
    write_ui_asset(&layout_path, DETACH_THEME_UI_LAYOUT_ASSET);
    write_ui_asset(&imported_theme_path, IMPORTED_THEME_COLLISION_ASSET);

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.zui", None)
        .expect("ui asset editor should open from project asset id");
    manager
        .select_ui_asset_editor_theme_source(&instance_id, 1)
        .expect("select imported theme");

    assert!(manager
        .clone_ui_asset_editor_selected_theme_source_to_local(&instance_id)
        .expect("clone selected imported theme into local layer"));

    let cloned = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after clone");
    assert_eq!(cloned.theme_selected_source_kind, "Local");
    assert_eq!(cloned.theme_selected_source_reference, "local");
    assert_eq!(
        cloned.theme_source_items,
        vec![
            "Local Theme • 3 tokens • 2 rules".to_string(),
            "res://ui/theme/shared_theme.zui • 2 tokens • 1 rules".to_string(),
        ]
    );
    assert_eq!(
        cloned.theme_selected_source_token_items,
        vec![
            "accent = \"#4488ff\"".to_string(),
            "panel = \"$shared_theme_accent\"".to_string(),
            "shared_theme_accent = \"#223344\"".to_string(),
        ]
    );

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save cloned theme ui asset");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved cloned ui asset");
    assert_eq!(
        document.imports.styles,
        vec!["res://ui/theme/shared_theme.zui".to_string()]
    );
    assert_eq!(
        document
            .tokens
            .get("shared_theme_accent")
            .and_then(toml::Value::as_str),
        Some("#223344")
    );
    assert_eq!(
        document
            .stylesheets
            .iter()
            .map(|sheet| sheet.id.as_str())
            .collect::<Vec<_>>(),
        vec!["shared_theme_local_theme", "local_theme"]
    );

    assert!(manager
        .undo_ui_asset_editor(&instance_id)
        .expect("undo clone imported theme"));
    let undone = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after undo clone");
    assert_eq!(undone.theme_selected_source_kind, "Imported");
    assert_eq!(
        undone.theme_selected_source_reference,
        "res://ui/theme/shared_theme.zui"
    );

    assert!(manager
        .redo_ui_asset_editor(&instance_id)
        .expect("redo clone imported theme"));
    let redone = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after redo clone");
    assert_eq!(redone.theme_selected_source_kind, "Local");
    assert_eq!(
        redone.theme_source_items,
        vec![
            "Local Theme • 3 tokens • 2 rules".to_string(),
            "res://ui/theme/shared_theme.zui • 2 tokens • 1 rules".to_string(),
        ]
    );

    let imported_theme_source =
        fs::read_to_string(&imported_theme_path).expect("imported theme source should remain");
    let imported_theme =
        UiZuiAssetLoader::load_zui_str(&imported_theme_source).expect("imported theme asset");
    assert_eq!(imported_theme.asset.id, "ui.theme.shared_theme");

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

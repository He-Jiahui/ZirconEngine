use std::fs;

use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;

use super::support::*;

#[test]
fn editor_manager_runs_ui_asset_style_class_editing_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_style_classes");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_style_classes_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select style target");
    manager
        .add_ui_asset_editor_class_to_selection(&instance_id, "toolbar")
        .expect("add toolbar class");
    manager
        .remove_ui_asset_editor_class_from_selection(&instance_id, "primary")
        .expect("remove primary class");

    let reflection = manager
        .ui_asset_editor_reflection(&instance_id)
        .expect("updated reflection");
    assert_eq!(reflection.style_inspector.classes, vec!["toolbar"]);
    assert!(reflection.source_dirty);

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    let button = document.node("button").expect("button node");
    assert_eq!(button.classes, vec!["toolbar".to_string()]);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_style_rule_editing_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_style_rules");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_style_rules_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select style target");
    manager
        .select_ui_asset_editor_stylesheet_rule(&instance_id, 1)
        .expect("select stylesheet rule");
    manager
        .rename_ui_asset_editor_selected_stylesheet_rule(&instance_id, "Button.toolbar:hover")
        .expect("rename stylesheet rule");
    manager
        .delete_ui_asset_editor_selected_stylesheet_rule(&instance_id)
        .expect("delete stylesheet rule");

    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert_eq!(pane.style_rule_items, vec![".primary".to_string()]);
    assert_eq!(pane.style_rule_selected_index, 0);
    assert_eq!(pane.style_selected_rule_selector, ".primary");
    assert!(pane.source_dirty);

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    let selectors = document.stylesheets[0]
        .rules
        .iter()
        .map(|rule| rule.selector.clone())
        .collect::<Vec<_>>();
    assert_eq!(selectors, vec![".primary".to_string()]);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_style_rule_declaration_editing_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_style_rule_declarations");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_style_rule_declarations_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select style target");
    manager
        .select_ui_asset_editor_stylesheet_rule(&instance_id, 0)
        .expect("select stylesheet rule");
    manager
        .select_ui_asset_editor_style_rule_declaration(&instance_id, 0)
        .expect("select declaration");
    manager
        .upsert_ui_asset_editor_selected_style_rule_declaration(&instance_id, "slot.padding", "6")
        .expect("rename declaration");
    manager
        .delete_ui_asset_editor_selected_style_rule_declaration(&instance_id)
        .expect("delete declaration");

    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert!(pane.style_rule_declaration_items.is_empty());
    assert_eq!(pane.style_rule_declaration_selected_index, -1);
    assert_eq!(pane.style_selected_rule_declaration_path, "");
    assert_eq!(pane.style_selected_rule_declaration_value, "");
    assert!(pane.source_dirty);

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    let rule = &document.stylesheets[0].rules[0];
    assert!(rule.set.self_values.is_empty());
    assert!(rule.set.slot.is_empty());

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_projects_matched_style_rule_summaries_into_stylesheet_items() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_matched_rule_details");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_matched_rule_details_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select style target");
    manager
        .toggle_ui_asset_editor_pseudo_state(&instance_id, "hover")
        .expect("toggle hover preview");

    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert!(pane
        .stylesheet_items
        .contains(&"selection selector: #SaveButton".to_string()));
    assert!(pane.stylesheet_items.contains(&"states: hover".to_string()));
    assert!(pane
        .stylesheet_items
        .contains(&".primary (spec 10, order 0)".to_string()));
    assert!(pane
        .stylesheet_items
        .contains(&".primary:hover (spec 20, order 1)".to_string()));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_projects_selected_matched_style_rule_details() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_matched_rule_details");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_matched_rule_details_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select style target");
    manager
        .toggle_ui_asset_editor_pseudo_state(&instance_id, "hover")
        .expect("toggle hover preview");
    manager
        .select_ui_asset_editor_matched_style_rule(&instance_id, 1)
        .expect("select matched style rule");

    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert_eq!(
        pane.style_matched_rule_items,
        vec![
            ".primary [editor.tests.asset.style::local]".to_string(),
            ".primary:hover [editor.tests.asset.style::local]".to_string()
        ]
    );
    assert_eq!(pane.style_matched_rule_selected_index, 1);
    assert_eq!(
        pane.style_selected_matched_rule_origin,
        "editor.tests.asset.style::local"
    );
    assert_eq!(pane.style_selected_matched_rule_selector, ".primary:hover");
    assert_eq!(pane.style_selected_matched_rule_specificity, 20);
    assert_eq!(pane.style_selected_matched_rule_source_order, 1);
    assert_eq!(
        pane.style_selected_matched_rule_declaration_items,
        vec!["self.text.color = \"#ffeeaa\"".to_string()]
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_widget_inspector_editing_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_widget_inspector");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_widget_inspector_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select style target");
    manager
        .set_ui_asset_editor_selected_widget_control_id(&instance_id, "ConfirmButton")
        .expect("set selected widget control id");
    manager
        .set_ui_asset_editor_selected_widget_text_property(&instance_id, "Confirm")
        .expect("set selected widget text property");

    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.inspector_widget_label, "Button");
    assert_eq!(pane.inspector_control_id, "ConfirmButton");
    assert_eq!(pane.inspector_text_prop, "Confirm");
    assert!(pane.source_dirty);

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    let button = document.node("button").expect("button node");
    assert_eq!(button.control_id.as_deref(), Some("ConfirmButton"));
    assert_eq!(
        button.props.get("text").and_then(toml::Value::as_str),
        Some("Confirm")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_slot_inspector_editing_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_slot_inspector");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_slot_inspector_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select style target");
    manager
        .set_ui_asset_editor_selected_slot_mount(&instance_id, "footer")
        .expect("set selected slot mount");
    manager
        .set_ui_asset_editor_selected_slot_padding(&instance_id, "12")
        .expect("set selected slot padding");
    manager
        .set_ui_asset_editor_selected_slot_width_preferred(&instance_id, "240")
        .expect("set selected slot width preferred");
    manager
        .set_ui_asset_editor_selected_slot_height_preferred(&instance_id, "44")
        .expect("set selected slot height preferred");

    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert_eq!(pane.inspector_mount, "footer");
    assert_eq!(pane.inspector_slot_padding, "12");
    assert_eq!(pane.inspector_slot_width_preferred, "240");
    assert_eq!(pane.inspector_slot_height_preferred, "44");
    assert!(pane.source_dirty);

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    let child_mount = document
        .node("root")
        .expect("root node")
        .children
        .iter()
        .find(|child_mount| child_mount.node.node_id == "button")
        .expect("button child mount");
    assert_eq!(child_mount.mount.as_deref(), Some("footer"));
    assert_eq!(
        slot_value(&child_mount.slot, &["padding"]).and_then(toml::Value::as_integer),
        Some(12)
    );
    assert_eq!(
        slot_value(&child_mount.slot, &["layout", "width", "preferred"])
            .and_then(toml::Value::as_integer),
        Some(240)
    );
    assert_eq!(
        slot_value(&child_mount.slot, &["layout", "height", "preferred"])
            .and_then(toml::Value::as_integer),
        Some(44)
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_layout_inspector_editing_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_layout_inspector");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_layout_inspector_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select style target");
    manager
        .set_ui_asset_editor_selected_layout_width_preferred(&instance_id, "220")
        .expect("set selected layout width preferred");
    manager
        .set_ui_asset_editor_selected_layout_height_preferred(&instance_id, "48")
        .expect("set selected layout height preferred");

    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert_eq!(pane.inspector_layout_width_preferred, "220");
    assert_eq!(pane.inspector_layout_height_preferred, "48");
    assert!(pane.source_dirty);

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    let button = document.node("button").expect("button node");
    assert_eq!(
        layout_value(button.layout.as_ref(), &["width", "preferred"])
            .and_then(toml::Value::as_integer),
        Some(220)
    );
    assert_eq!(
        layout_value(button.layout.as_ref(), &["height", "preferred"])
            .and_then(toml::Value::as_integer),
        Some(48)
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_parent_specific_semantic_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_semantic_inspector");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_semantic_inspector_file").join("semantic.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, SEMANTIC_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select semantic target");
    manager
        .select_ui_asset_editor_slot_semantic(&instance_id, 0)
        .expect("select slot semantic");
    manager
        .set_ui_asset_editor_selected_slot_semantic_value(&instance_id, "0.5")
        .expect("set selected slot semantic value");
    manager
        .select_ui_asset_editor_layout_semantic(&instance_id, 0)
        .expect("select layout semantic");
    manager
        .set_ui_asset_editor_selected_layout_semantic_value(&instance_id, "Horizontal")
        .expect("set selected layout semantic value");

    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert_eq!(pane.inspector_slot_semantic_title, "Overlay Slot");
    assert_eq!(pane.inspector_slot_semantic_selected_index, 0);
    assert_eq!(pane.inspector_slot_semantic_path, "layout.anchor.x");
    assert_eq!(pane.inspector_slot_semantic_value, "0.5");
    assert_eq!(pane.inspector_layout_semantic_title, "Scrollable Layout");
    assert_eq!(pane.inspector_layout_semantic_selected_index, 0);
    assert_eq!(pane.inspector_layout_semantic_path, "container.axis");
    assert_eq!(pane.inspector_layout_semantic_value, "\"Horizontal\"");
    assert!(pane.source_dirty);

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    let child_mount = document
        .node("root")
        .expect("root node")
        .children
        .iter()
        .find(|child_mount| child_mount.node.node_id == "scroll_panel")
        .expect("scroll panel child mount");
    assert_eq!(
        slot_value(&child_mount.slot, &["layout", "anchor", "x"]).and_then(toml::Value::as_float),
        Some(0.5)
    );
    let scroll_panel = document.node("scroll_panel").expect("scroll panel");
    assert_eq!(
        layout_value(scroll_panel.layout.as_ref(), &["container", "axis"])
            .and_then(toml::Value::as_str),
        Some("Horizontal")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_style_token_editing_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_style_tokens");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_style_tokens_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");
    manager
        .upsert_ui_asset_editor_style_token(&instance_id, "surface_fill", "#223344")
        .expect("add token");
    manager
        .select_ui_asset_editor_style_token(&instance_id, 0)
        .expect("select token");
    manager
        .upsert_ui_asset_editor_style_token(&instance_id, "accent_primary", "#99bbff")
        .expect("rename token");
    manager
        .delete_ui_asset_editor_selected_style_token(&instance_id)
        .expect("delete selected token");

    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert_eq!(
        pane.style_token_items,
        vec![
            "panel_gap = 12".to_string(),
            "surface_fill = \"#223344\"".to_string()
        ]
    );
    assert_eq!(pane.style_token_selected_index, 0);
    assert_eq!(pane.style_selected_token_name, "panel_gap");
    assert_eq!(pane.style_selected_token_value, "12");

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    assert!(!document.tokens.contains_key("accent"));
    assert!(!document.tokens.contains_key("accent_primary"));
    assert_eq!(
        document
            .tokens
            .get("surface_fill")
            .and_then(toml::Value::as_str),
        Some("#223344")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

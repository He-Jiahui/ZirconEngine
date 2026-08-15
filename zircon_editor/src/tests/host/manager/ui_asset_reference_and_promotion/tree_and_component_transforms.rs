use super::*;

#[test]
fn editor_manager_runs_ui_asset_reparent_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_tree_reparent");
    let ui_asset_path = unique_temp_dir("zircon_editor_asset_tree_reparent_file").join("tree.zui");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, TREE_REPARENT_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");

    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 3)
        .expect("select loose node");
    assert!(
        manager
            .reparent_ui_asset_editor_selected_node_into_previous(&instance_id)
            .expect("reparent into previous sibling")
    );
    let previous = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("previous pane");
    assert_eq!(previous.inspector_selected_node_id, "loose");
    assert_eq!(previous.inspector_parent_node_id, "group_a");

    assert!(
        manager
            .reparent_ui_asset_editor_selected_node_outdent(&instance_id)
            .expect("outdent node")
    );
    let outdented = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("outdented pane");
    assert_eq!(outdented.inspector_selected_node_id, "loose");
    assert_eq!(outdented.inspector_parent_node_id, "root");

    assert!(
        manager
            .reparent_ui_asset_editor_selected_node_into_next(&instance_id)
            .expect("reparent into next sibling")
    );
    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    assert_eq!(
        document.node("group_b").map(|node| node
            .children
            .iter()
            .map(|child| child.node.node_id.clone())
            .collect::<Vec<_>>()),
        Some(vec!["loose".to_string(), "nested_b".to_string()])
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_converts_selected_ui_asset_node_to_reference_from_palette_selection() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_convert_reference");
    let project_root = unique_temp_dir("zircon_editor_asset_convert_reference_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);

    let widget_path = project_root
        .join("assets")
        .join("ui")
        .join("widgets")
        .join("toolbar_button.zui");
    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.zui");
    fs::create_dir_all(widget_path.parent().unwrap()).unwrap();
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    write_ui_asset(
        &widget_path,
        r##"
[asset]
kind = "widget"
id = "ui.widgets.toolbar_button"
version = 1
display_name = "Toolbar Button"

[root]
node = "button_root"

[components.ToolbarButton]
root = "button_root"

[components.ToolbarButton.params.text]
type = "string"
default = "Toolbar"

[nodes.button_root]
kind = "native"
type = "Button"
control_id = "ToolbarButton"
props = { text = "$param.text" }
"##,
    );
    write_ui_asset(
        &layout_path,
        r##"
[asset]
kind = "layout"
id = "ui.layouts.editor"
version = 1
display_name = "Editor Layout"

[imports]
widgets = ["res://ui/widgets/toolbar_button.zui#ToolbarButton"]

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
classes = ["primary"]
props = { text = "Save" }
style_overrides = { self = { text = { color = "#ffffff" } }, slot = { padding = 4 } }
"##,
    );

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.zui", None)
        .expect("ui asset editor should open from project asset id");
    let palette_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("editor pane")
        .palette_items
        .iter()
        .position(|item| item == "Reference / ToolbarButton")
        .expect("toolbar button palette item");

    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select button");
    manager
        .select_ui_asset_editor_palette_index(&instance_id, palette_index)
        .expect("select toolbar button reference palette item");
    assert!(
        manager
            .ui_asset_editor_pane_presentation(&instance_id)
            .expect("pane after palette selection")
            .can_convert_to_reference
    );

    assert!(
        manager
            .convert_ui_asset_editor_selected_node_to_reference(&instance_id)
            .expect("convert selected node to reference")
    );

    let converted = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("converted pane");
    assert!(converted.can_open_reference);
    assert!(!converted.can_convert_to_reference);

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    let button = document.node("button").expect("button node");
    assert_eq!(button.kind, UiNodeDefinitionKind::Reference);
    assert_eq!(
        button.component_ref.as_deref(),
        Some("res://ui/widgets/toolbar_button.zui#ToolbarButton")
    );
    assert_eq!(
        button.params.get("text").and_then(toml::Value::as_str),
        Some("Save")
    );

    let opened = manager
        .open_ui_asset_editor_selected_reference(&instance_id)
        .expect("open selected reference")
        .expect("reference editor instance");
    let reflection = manager
        .ui_asset_editor_reflection(&opened)
        .expect("reference reflection");
    assert_eq!(
        reflection.route.asset_id,
        "res://ui/widgets/toolbar_button.zui"
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_extracts_selected_ui_asset_node_to_local_component() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_extract_component");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_extract_component_file").join("style.zui");
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
        .expect("select button");
    assert!(
        manager
            .ui_asset_editor_pane_presentation(&instance_id)
            .expect("pane before extract")
            .can_extract_component
    );

    assert!(
        manager
            .extract_ui_asset_editor_selected_node_to_component(&instance_id)
            .expect("extract selected node to local component")
    );

    let extracted = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after extract");
    assert_eq!(extracted.inspector_selected_node_id, "button");
    assert_eq!(extracted.inspector_widget_kind, "Component");
    assert_eq!(extracted.inspector_widget_label, "SaveButton");
    assert!(
        extracted
            .palette_items
            .iter()
            .any(|item| item == "Component / SaveButton")
    );

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    let component = document
        .components
        .get("SaveButton")
        .expect("new local component");
    assert_eq!(
        document
            .node("button")
            .and_then(|node| node.component.as_deref()),
        Some("SaveButton")
    );
    assert_eq!(
        document
            .node(&component.root.node_id)
            .and_then(|node| node.widget_type.as_deref()),
        Some("Button")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

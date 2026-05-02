use std::fs;

use zircon_runtime::asset::assets::{UiStyleAsset, UiWidgetAsset};
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::ui::template::{UiAssetKind, UiNodeDefinitionKind};

use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::workbench::project::EditorProjectDocument;

use super::support::*;

#[test]
fn editor_manager_opens_selected_ui_asset_reference_in_new_editor_instance() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_open_reference");
    let project_root = unique_temp_dir("zircon_editor_asset_open_reference_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();

    let widget_path = project_root
        .join("assets")
        .join("ui")
        .join("widgets")
        .join("button.ui.toml");
    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.ui.toml");
    fs::create_dir_all(widget_path.parent().unwrap()).unwrap();
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    write_ui_asset(
        &widget_path,
        r#"
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
"#,
    );
    write_ui_asset(
        &layout_path,
        r#"
[asset]
kind = "layout"
id = "ui.layouts.editor"
version = 1
display_name = "Editor Layout"

[imports]
widgets = ["res://ui/widgets/button.ui.toml#ToolbarButton"]

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "toolbar" }]

[nodes.toolbar]
kind = "reference"
component_ref = "res://ui/widgets/button.ui.toml#ToolbarButton"
control_id = "ToolbarHost"
"#,
    );

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .expect("ui asset editor should open from project asset id");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select toolbar reference");

    let opened = manager
        .open_ui_asset_editor_selected_reference(&instance_id)
        .expect("open selected reference")
        .expect("reference view instance");

    let reflection = manager
        .ui_asset_editor_reflection(&opened)
        .expect("reference reflection");
    assert_eq!(reflection.route.asset_id, "res://ui/widgets/button.ui.toml");
    assert_eq!(reflection.display_name, "Toolbar Button");
    assert_eq!(reflection.route.asset_kind, UiAssetKind::Widget);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_activates_selected_ui_asset_reference_from_hierarchy() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_activate_reference");
    let project_root = unique_temp_dir("zircon_editor_asset_activate_reference_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();

    let widget_path = project_root
        .join("assets")
        .join("ui")
        .join("widgets")
        .join("button.ui.toml");
    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.ui.toml");
    fs::create_dir_all(widget_path.parent().unwrap()).unwrap();
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    write_ui_asset(
        &widget_path,
        r#"
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
"#,
    );
    write_ui_asset(
        &layout_path,
        r#"
[asset]
kind = "layout"
id = "ui.layouts.editor"
version = 1
display_name = "Editor Layout"

[imports]
widgets = ["res://ui/widgets/button.ui.toml#ToolbarButton"]

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "toolbar" }]

[nodes.toolbar]
kind = "reference"
component_ref = "res://ui/widgets/button.ui.toml#ToolbarButton"
control_id = "ToolbarHost"
"#,
    );

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .expect("ui asset editor should open from project asset id");

    let opened = manager
        .activate_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("activate hierarchy item")
        .expect("reference view instance");

    let reflection = manager
        .ui_asset_editor_reflection(&opened)
        .expect("reference reflection");
    assert_eq!(reflection.route.asset_id, "res://ui/widgets/button.ui.toml");
    assert_eq!(reflection.display_name, "Toolbar Button");
    assert_eq!(reflection.route.asset_kind, UiAssetKind::Widget);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_activates_selected_ui_asset_reference_from_preview() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_activate_preview_reference");
    let project_root = unique_temp_dir("zircon_editor_asset_activate_preview_reference_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();

    let widget_path = project_root
        .join("assets")
        .join("ui")
        .join("widgets")
        .join("button.ui.toml");
    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.ui.toml");
    fs::create_dir_all(widget_path.parent().unwrap()).unwrap();
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    write_ui_asset(
        &widget_path,
        r#"
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
"#,
    );
    write_ui_asset(
        &layout_path,
        r#"
[asset]
kind = "layout"
id = "ui.layouts.editor"
version = 1
display_name = "Editor Layout"

[imports]
widgets = ["res://ui/widgets/button.ui.toml#ToolbarButton"]

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "toolbar" }]

[nodes.toolbar]
kind = "reference"
component_ref = "res://ui/widgets/button.ui.toml#ToolbarButton"
control_id = "ToolbarHost"
"#,
    );

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .expect("ui asset editor should open from project asset id");
    let preview_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("editor pane")
        .preview_items
        .iter()
        .position(|item| item.contains("ToolbarHost"))
        .expect("toolbar host preview item");

    let opened = manager
        .activate_ui_asset_editor_preview_index(&instance_id, preview_index)
        .expect("activate preview reference")
        .expect("reference view instance");

    let reflection = manager
        .ui_asset_editor_reflection(&opened)
        .expect("reference reflection");
    assert_eq!(reflection.route.asset_id, "res://ui/widgets/button.ui.toml");
    assert_eq!(reflection.display_name, "Toolbar Button");
    assert_eq!(reflection.route.asset_kind, UiAssetKind::Widget);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_runs_ui_asset_reparent_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_tree_reparent");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_tree_reparent_file").join("tree.ui.toml");
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
    assert!(manager
        .reparent_ui_asset_editor_selected_node_into_previous(&instance_id)
        .expect("reparent into previous sibling"));
    let previous = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("previous pane");
    assert_eq!(previous.inspector_selected_node_id, "loose");
    assert_eq!(previous.inspector_parent_node_id, "group_a");

    assert!(manager
        .reparent_ui_asset_editor_selected_node_outdent(&instance_id)
        .expect("outdent node"));
    let outdented = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("outdented pane");
    assert_eq!(outdented.inspector_selected_node_id, "loose");
    assert_eq!(outdented.inspector_parent_node_id, "root");

    assert!(manager
        .reparent_ui_asset_editor_selected_node_into_next(&instance_id)
        .expect("reparent into next sibling"));
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
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();

    let widget_path = project_root
        .join("assets")
        .join("ui")
        .join("widgets")
        .join("toolbar_button.ui.toml");
    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.ui.toml");
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
widgets = ["res://ui/widgets/toolbar_button.ui.toml#ToolbarButton"]

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
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
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

    assert!(manager
        .convert_ui_asset_editor_selected_node_to_reference(&instance_id)
        .expect("convert selected node to reference"));

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
        Some("res://ui/widgets/toolbar_button.ui.toml#ToolbarButton")
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
        "res://ui/widgets/toolbar_button.ui.toml"
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
        unique_temp_dir("zircon_editor_asset_extract_component_file").join("style.ui.toml");
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

    assert!(manager
        .extract_ui_asset_editor_selected_node_to_component(&instance_id)
        .expect("extract selected node to local component"));

    let extracted = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after extract");
    assert_eq!(extracted.inspector_selected_node_id, "button");
    assert_eq!(extracted.inspector_widget_kind, "Component");
    assert_eq!(extracted.inspector_widget_label, "SaveButton");
    assert!(extracted
        .palette_items
        .iter()
        .any(|item| item == "Component / SaveButton"));

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

#[test]
fn editor_manager_promotes_selected_ui_asset_component_to_external_widget_asset() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_promote_widget");
    let project_root = unique_temp_dir("zircon_editor_asset_promote_widget_project");
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
        .join("editor.ui.toml");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    write_ui_asset(&layout_path, STYLE_UI_LAYOUT_ASSET);

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
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
        .join("save_button.ui.toml");
    let widget_source = fs::read_to_string(&widget_path).expect("promoted widget file");
    let widget_asset = UiWidgetAsset::from_toml_str(&widget_source).expect("widget asset");
    assert_eq!(widget_asset.document.asset.id, "ui.widgets.save_button");
    assert_eq!(
        widget_asset
            .document
            .root
            .as_ref()
            .map(|root| root.node_id.as_str()),
        Some("savebutton_root")
    );
    assert!(widget_asset.document.components.contains_key("SaveButton"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    let button = document.node("button").expect("button node");
    assert_eq!(button.kind, UiNodeDefinitionKind::Reference);
    assert_eq!(
        button.component_ref.as_deref(),
        Some("res://ui/widgets/save_button.ui.toml#SaveButton")
    );
    assert!(document
        .imports
        .widgets
        .iter()
        .any(|reference| { reference == "res://ui/widgets/save_button.ui.toml#SaveButton" }));
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
        UiWidgetAsset::from_toml_str(&redone_widget_source).expect("redone widget asset");
    assert_eq!(redone_widget.document.asset.id, "ui.widgets.save_button");

    let opened = manager
        .open_ui_asset_editor_selected_reference(&instance_id)
        .expect("open promoted reference")
        .expect("reference editor instance");
    let reflection = manager
        .ui_asset_editor_reflection(&opened)
        .expect("promoted widget reflection");
    assert_eq!(
        reflection.route.asset_id,
        "res://ui/widgets/save_button.ui.toml"
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

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
        .join("editor.ui.toml");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    write_ui_asset(&layout_path, STYLE_UI_LAYOUT_ASSET);

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
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
        .join("editor_theme.ui.toml");
    let theme_source = fs::read_to_string(&theme_path).expect("promoted theme file");
    let theme_asset = UiStyleAsset::from_toml_str(&theme_source).expect("style asset");
    assert_eq!(theme_asset.document.asset.id, "ui.theme.editor_theme");
    assert_eq!(
        theme_asset.document.asset.display_name,
        "Styled UI Asset Theme"
    );
    assert_eq!(
        theme_asset
            .document
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
        "res://ui/themes/editor_theme.ui.toml"
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
        vec!["res://ui/themes/editor_theme.ui.toml".to_string()]
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
        "res://ui/themes/editor_theme.ui.toml"
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
        "res://ui/themes/editor_theme.ui.toml"
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
        .join("editor.ui.toml");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    write_ui_asset(&layout_path, STYLE_UI_LAYOUT_ASSET);

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .expect("ui asset editor should open from project asset id");
    manager
        .set_ui_asset_editor_promote_theme_asset_id(
            &instance_id,
            "res://ui/themes/custom/editor_shell.ui.toml",
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
        .join("editor_shell.ui.toml");
    let theme_source = fs::read_to_string(&theme_path).expect("custom promoted theme file");
    let theme_asset = UiStyleAsset::from_toml_str(&theme_source).expect("custom style asset");
    assert_eq!(
        theme_asset.document.asset.id,
        "ui.theme.custom.editor_shell"
    );
    assert_eq!(
        theme_asset.document.asset.display_name,
        "Editor Shell Theme"
    );

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor after custom theme promote");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    assert_eq!(
        document.imports.styles,
        vec!["res://ui/themes/custom/editor_shell.ui.toml".to_string()]
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
        .join("editor.ui.toml");
    let imported_theme_path = project_root
        .join("assets")
        .join("ui")
        .join("theme")
        .join("shared_theme.ui.toml");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    fs::create_dir_all(imported_theme_path.parent().unwrap()).unwrap();
    write_ui_asset(&layout_path, DETACH_THEME_UI_LAYOUT_ASSET);
    write_ui_asset(&imported_theme_path, IMPORTED_THEME_COLLISION_ASSET);

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
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
        "res://ui/theme/shared_theme.ui.toml"
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
        "res://ui/theme/shared_theme.ui.toml"
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
        UiStyleAsset::from_toml_str(&imported_theme_source).expect("imported theme asset");
    assert_eq!(imported_theme.document.asset.id, "ui.theme.shared_theme");

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
        .join("editor.ui.toml");
    let imported_theme_path = project_root
        .join("assets")
        .join("ui")
        .join("theme")
        .join("shared_theme.ui.toml");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    fs::create_dir_all(imported_theme_path.parent().unwrap()).unwrap();
    write_ui_asset(&layout_path, DETACH_THEME_UI_LAYOUT_ASSET);
    write_ui_asset(&imported_theme_path, IMPORTED_THEME_COLLISION_ASSET);

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
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
            "res://ui/theme/shared_theme.ui.toml • 2 tokens • 1 rules".to_string(),
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
        vec!["res://ui/theme/shared_theme.ui.toml".to_string()]
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
        "res://ui/theme/shared_theme.ui.toml"
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
            "res://ui/theme/shared_theme.ui.toml • 2 tokens • 1 rules".to_string(),
        ]
    );

    let imported_theme_source =
        fs::read_to_string(&imported_theme_path).expect("imported theme source should remain");
    let imported_theme =
        UiStyleAsset::from_toml_str(&imported_theme_source).expect("imported theme asset");
    assert_eq!(imported_theme.document.asset.id, "ui.theme.shared_theme");

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
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();

    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.ui.toml");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    write_ui_asset(&layout_path, STYLE_UI_LAYOUT_ASSET);

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
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
            "res://ui/widgets/custom/editor_save.ui.toml",
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
        .join("editor_save.ui.toml");
    let widget_source = fs::read_to_string(&widget_path).expect("custom promoted widget file");
    let widget_asset = UiWidgetAsset::from_toml_str(&widget_source).expect("widget asset");
    assert_eq!(
        widget_asset.document.asset.id,
        "ui.widgets.custom.editor_save"
    );
    assert!(widget_asset
        .document
        .components
        .contains_key("EditorSaveButton"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    let button = document.node("button").expect("button node");
    assert_eq!(
        button.component_ref.as_deref(),
        Some("res://ui/widgets/custom/editor_save.ui.toml#EditorSaveButton")
    );
    assert!(document.imports.widgets.iter().any(|reference| {
        reference == "res://ui/widgets/custom/editor_save.ui.toml#EditorSaveButton"
    }));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

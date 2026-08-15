use super::*;

#[test]
fn editor_manager_opens_selected_ui_asset_reference_in_new_editor_instance() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_open_reference");
    let project_root = unique_temp_dir("zircon_editor_asset_open_reference_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);

    let widget_path = project_root
        .join("assets")
        .join("ui")
        .join("widgets")
        .join("button.zui");
    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.zui");
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
widgets = ["res://ui/widgets/button.zui#ToolbarButton"]

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "toolbar" }]

[nodes.toolbar]
kind = "reference"
component_ref = "res://ui/widgets/button.zui#ToolbarButton"
control_id = "ToolbarHost"
"#,
    );

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.zui", None)
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
    assert_eq!(reflection.route.asset_id, "res://ui/widgets/button.zui");
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
    create_project_with_default_world(&project_root);

    let widget_path = project_root
        .join("assets")
        .join("ui")
        .join("widgets")
        .join("button.zui");
    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.zui");
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
widgets = ["res://ui/widgets/button.zui#ToolbarButton"]

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "toolbar" }]

[nodes.toolbar]
kind = "reference"
component_ref = "res://ui/widgets/button.zui#ToolbarButton"
control_id = "ToolbarHost"
"#,
    );

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.zui", None)
        .expect("ui asset editor should open from project asset id");

    let opened = manager
        .activate_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("activate hierarchy item")
        .expect("reference view instance");

    let reflection = manager
        .ui_asset_editor_reflection(&opened)
        .expect("reference reflection");
    assert_eq!(reflection.route.asset_id, "res://ui/widgets/button.zui");
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
    create_project_with_default_world(&project_root);

    let widget_path = project_root
        .join("assets")
        .join("ui")
        .join("widgets")
        .join("button.zui");
    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.zui");
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
widgets = ["res://ui/widgets/button.zui#ToolbarButton"]

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "toolbar" }]

[nodes.toolbar]
kind = "reference"
component_ref = "res://ui/widgets/button.zui#ToolbarButton"
control_id = "ToolbarHost"
"#,
    );

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.zui", None)
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
    assert_eq!(reflection.route.asset_id, "res://ui/widgets/button.zui");
    assert_eq!(reflection.display_name, "Toolbar Button");
    assert_eq!(reflection.route.asset_kind, UiAssetKind::Widget);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

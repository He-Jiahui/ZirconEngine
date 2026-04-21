use std::fs;

use crate::ui::asset_editor::UiAssetEditorMode;
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;

use super::support::*;

#[test]
fn editor_manager_runs_ui_asset_binding_inspector_editing_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_binding_inspector");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_binding_inspector_file").join("style.ui.toml");
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
        .add_ui_asset_editor_binding(&instance_id)
        .expect("add binding");
    manager
        .set_ui_asset_editor_selected_binding_id(&instance_id, "SaveButton/onHover")
        .expect("set selected binding id");
    manager
        .set_ui_asset_editor_selected_binding_event(&instance_id, "onHover")
        .expect("set selected binding event");
    manager
        .set_ui_asset_editor_selected_binding_route(&instance_id, "MenuAction.HighlightSave")
        .expect("set selected binding route");

    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert_eq!(pane.inspector_binding_selected_index, 0);
    assert_eq!(pane.inspector_binding_id, "SaveButton/onHover");
    assert_eq!(pane.inspector_binding_event, "onHover");
    assert_eq!(pane.inspector_binding_route, "MenuAction.HighlightSave");
    assert!(pane.source_dirty);

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    let button = document.node("button").expect("button node");
    assert_eq!(button.bindings.len(), 1);
    assert_eq!(button.bindings[0].id, "SaveButton/onHover");
    assert_eq!(button.bindings[0].event.to_string(), "onHover");
    assert_eq!(
        button.bindings[0].route.as_deref(),
        Some("MenuAction.HighlightSave")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_structured_binding_inspector_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_structured_binding_inspector");
    let ui_asset_path = unique_temp_dir("zircon_editor_asset_structured_binding_inspector_file")
        .join("structured-binding.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STRUCTURED_BINDING_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select binding target");
    manager
        .select_ui_asset_editor_binding_event_option(&instance_id, 1)
        .expect("select double click event");
    manager
        .select_ui_asset_editor_binding_action_kind(&instance_id, 2)
        .expect("select action kind");
    manager
        .set_ui_asset_editor_selected_binding_route(&instance_id, "EditorActions.SaveProject")
        .expect("set action target");
    manager
        .select_ui_asset_editor_binding_payload(&instance_id, 1)
        .expect("select mode payload");
    manager
        .upsert_ui_asset_editor_selected_binding_payload(&instance_id, "mode", "\"compact\"")
        .expect("update payload");
    manager
        .upsert_ui_asset_editor_selected_binding_payload(&instance_id, "channel", "\"toolbar\"")
        .expect("add payload");
    manager
        .delete_ui_asset_editor_selected_binding_payload(&instance_id)
        .expect("delete selected payload");

    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert_eq!(pane.inspector_binding_event, "onDoubleClick");
    assert_eq!(pane.inspector_binding_event_selected_index, 1);
    assert_eq!(
        pane.inspector_binding_event_items,
        vec![
            "onClick".to_string(),
            "onDoubleClick".to_string(),
            "onHover".to_string(),
            "onPress".to_string(),
            "onRelease".to_string(),
            "onChange".to_string(),
            "onSubmit".to_string(),
            "onToggle".to_string(),
            "onFocus".to_string(),
            "onBlur".to_string(),
            "onScroll".to_string(),
            "onResize".to_string(),
            "onDragBegin".to_string(),
            "onDragUpdate".to_string(),
            "onDragEnd".to_string(),
        ]
    );
    assert_eq!(
        pane.inspector_binding_action_kind_items,
        vec![
            "None".to_string(),
            "Route".to_string(),
            "Action".to_string(),
        ]
    );
    assert_eq!(pane.inspector_binding_action_kind_selected_index, 2);
    assert_eq!(pane.inspector_binding_route, "EditorActions.SaveProject");
    assert_eq!(
        pane.inspector_binding_payload_items,
        vec![
            "confirm = true".to_string(),
            "mode = \"compact\"".to_string(),
        ]
    );
    assert_eq!(pane.inspector_binding_payload_selected_index, 0);
    assert_eq!(pane.inspector_binding_payload_key, "confirm");
    assert_eq!(pane.inspector_binding_payload_value, "true");
    assert!(pane.source_dirty);

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    let button = document.node("button").expect("button node");
    assert_eq!(button.bindings[0].event.to_string(), "onDoubleClick");
    assert!(button.bindings[0].route.is_none());
    let action = button.bindings[0].action.as_ref().expect("binding action");
    assert_eq!(action.action.as_deref(), Some("EditorActions.SaveProject"));
    assert_eq!(
        action.payload.get("mode").and_then(toml::Value::as_str),
        Some("compact")
    );
    assert!(action.payload.get("channel").is_none());

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_relative_nested_binding_payload_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_relative_nested_binding_payload");
    let ui_asset_path = unique_temp_dir("zircon_editor_asset_relative_nested_binding_payload_file")
        .join("relative-binding.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STRUCTURED_BINDING_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select binding target");
    manager
        .upsert_ui_asset_editor_selected_binding_payload(&instance_id, "context", "{}")
        .expect("upsert root context payload");

    let context_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after context insert")
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context = {}"))
        .expect("context payload index");
    manager
        .select_ui_asset_editor_binding_payload(&instance_id, context_index)
        .expect("select context payload");
    manager
        .upsert_ui_asset_editor_selected_binding_payload(&instance_id, "dialog.title", "\"Dialog\"")
        .expect("upsert relative object payload");

    let context_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after dialog insert")
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context = { dialog = { title = \"Dialog\" } }"))
        .expect("updated context payload index");
    manager
        .select_ui_asset_editor_binding_payload(&instance_id, context_index)
        .expect("reselect updated context payload");
    manager
        .upsert_ui_asset_editor_selected_binding_payload(&instance_id, "steps", "[]")
        .expect("upsert steps payload");

    let steps_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after steps insert")
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context.steps = []"))
        .expect("steps payload index");
    manager
        .select_ui_asset_editor_binding_payload(&instance_id, steps_index)
        .expect("select steps payload");
    manager
        .upsert_ui_asset_editor_selected_binding_payload(&instance_id, "0.label", "\"Queued\"")
        .expect("upsert relative collection payload");

    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert!(pane
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.dialog.title = \"Dialog\"")));
    assert!(pane
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.steps[0].label = \"Queued\"")));
    assert_eq!(
        pane.inspector_binding_payload_key,
        "context.steps[0].label".to_string()
    );
    assert_eq!(
        pane.inspector_binding_payload_value,
        "\"Queued\"".to_string()
    );
    assert!(pane.source_dirty);

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    let button = document.node("button").expect("button node");
    let action = button.bindings[0].action.as_ref().expect("binding action");
    let context = action
        .payload
        .get("context")
        .and_then(toml::Value::as_table)
        .expect("context payload");
    let dialog = context
        .get("dialog")
        .and_then(toml::Value::as_table)
        .expect("dialog payload");
    assert_eq!(
        dialog.get("title").and_then(toml::Value::as_str),
        Some("Dialog")
    );
    let steps = context
        .get("steps")
        .and_then(toml::Value::as_array)
        .expect("steps payload");
    assert_eq!(steps.len(), 1);
    assert_eq!(
        steps[0]
            .as_table()
            .and_then(|step| step.get("label"))
            .and_then(toml::Value::as_str),
        Some("Queued")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_binding_payload_suggestion_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_binding_payload_suggestion");
    let ui_asset_path = unique_temp_dir("zircon_editor_asset_binding_payload_suggestion_file")
        .join("layout.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STRUCTURED_BINDING_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, Some(UiAssetEditorMode::Design))
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select button");

    let initial = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("initial pane");
    assert_eq!(
        initial.inspector_binding_payload_suggestion_items,
        vec![
            "confirm = true".to_string(),
            "channel = \"toolbar\"".to_string(),
            "source = \"ui.click\"".to_string(),
        ]
    );

    manager
        .apply_ui_asset_editor_selected_binding_payload_suggestion(&instance_id, 2)
        .expect("apply binding payload suggestion");

    let updated = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert_eq!(
        updated.inspector_binding_payload_items,
        vec![
            "confirm = true".to_string(),
            "mode = \"full\"".to_string(),
            "source = \"ui.click\"".to_string(),
        ]
    );
    assert!(updated.source_dirty);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_binding_payload_suggestions_relative_to_selected_container() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_binding_payload_relative_suggestion");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_binding_payload_relative_suggestion_file")
            .join("layout.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(
        &ui_asset_path,
        CONTEXTUAL_BINDING_SUGGESTION_UI_LAYOUT_ASSET,
    );

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, Some(UiAssetEditorMode::Design))
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 2)
        .expect("select button");

    let initial = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("initial pane");
    assert_eq!(
        initial.inspector_binding_payload_suggestion_items,
        vec![
            "value = \"preview\"".to_string(),
            "committed = true".to_string(),
            "fields = [\"title\"]".to_string(),
            "context = { source = \"ui.click\", subject = \"field\" }".to_string(),
        ]
    );

    manager
        .upsert_ui_asset_editor_selected_binding_payload(&instance_id, "context", "{}")
        .expect("upsert context payload");
    let context_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after context insert")
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context = {}"))
        .expect("context payload");
    manager
        .select_ui_asset_editor_binding_payload(&instance_id, context_index)
        .expect("select context payload");

    let context_pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after context select");
    assert_eq!(
        context_pane.inspector_binding_payload_suggestion_items,
        vec![
            "source = \"ui.click\"".to_string(),
            "subject = \"field\"".to_string(),
        ]
    );

    manager
        .apply_ui_asset_editor_selected_binding_payload_suggestion(&instance_id, 1)
        .expect("apply contextual object suggestion");
    let after_object = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after contextual object apply");
    assert!(after_object
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.subject = \"field\"")));
    assert_eq!(
        after_object.inspector_binding_payload_key,
        "context.subject".to_string()
    );

    manager
        .upsert_ui_asset_editor_selected_binding_payload(&instance_id, "fields", "[]")
        .expect("upsert fields payload");
    let fields_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after fields insert")
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("fields = []"))
        .expect("fields payload");
    manager
        .select_ui_asset_editor_binding_payload(&instance_id, fields_index)
        .expect("select fields payload");

    let fields_pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after fields select");
    assert_eq!(
        fields_pane.inspector_binding_payload_suggestion_items,
        vec!["[0] = \"title\"".to_string(), "[1] = \"title\"".to_string()]
    );

    manager
        .apply_ui_asset_editor_selected_binding_payload_suggestion(&instance_id, 0)
        .expect("apply contextual collection suggestion");
    let after_collection = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane after contextual collection apply");
    assert!(after_collection
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("fields[0] = \"title\"")));
    assert_eq!(
        after_collection.inspector_binding_payload_key,
        "fields[0]".to_string()
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_palette_and_tree_authoring_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_tree_authoring");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_tree_authoring_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");
    let palette_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("initial pane")
        .palette_items
        .iter()
        .position(|item| item == "Native / Button")
        .expect("button palette item");

    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 0)
        .expect("select root");
    assert!(manager
        .select_ui_asset_editor_palette_index(&instance_id, palette_index)
        .expect("select palette item"));
    assert!(manager
        .insert_ui_asset_editor_selected_palette_item_as_child(&instance_id)
        .expect("insert palette child"));

    let inserted = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert_eq!(inserted.palette_selected_index, palette_index as i32);
    assert!(inserted
        .hierarchy_items
        .iter()
        .any(|item| item.contains("button_2")));

    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select original button");
    assert!(manager
        .wrap_ui_asset_editor_selected_node(&instance_id, "VerticalBox")
        .expect("wrap selected node"));
    assert!(manager
        .unwrap_ui_asset_editor_selected_node(&instance_id)
        .expect("unwrap selected node"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    assert!(document.contains_node("button_2"));
    assert_eq!(
        document.node("root").map(|node| node
            .children
            .iter()
            .map(|child| child.node.node_id.clone())
            .collect::<Vec<_>>()),
        Some(vec!["button".to_string(), "button_2".to_string()])
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_restores_ui_asset_tree_selection_across_undo_and_redo() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_tree_selection_undo");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_tree_selection_undo_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");
    let palette_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("initial pane")
        .palette_items
        .iter()
        .position(|item| item == "Native / Button")
        .expect("button palette item");

    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 0)
        .expect("select root");
    assert!(manager
        .select_ui_asset_editor_palette_index(&instance_id, palette_index)
        .expect("select palette item"));
    assert!(manager
        .insert_ui_asset_editor_selected_palette_item_as_child(&instance_id)
        .expect("insert palette child"));

    let inserted = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("inserted pane");
    assert_eq!(inserted.inspector_selected_node_id, "button_2");
    assert_eq!(inserted.source_selected_block_label, "[nodes.button_2]");

    assert!(manager
        .undo_ui_asset_editor(&instance_id)
        .expect("undo tree edit"));
    let undone = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("undone pane");
    assert_eq!(undone.inspector_selected_node_id, "root");
    assert_eq!(undone.source_selected_block_label, "[nodes.root]");

    assert!(manager
        .redo_ui_asset_editor(&instance_id)
        .expect("redo tree edit"));
    let redone = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("redone pane");
    assert_eq!(redone.inspector_selected_node_id, "button_2");
    assert_eq!(redone.source_selected_block_label, "[nodes.button_2]");

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

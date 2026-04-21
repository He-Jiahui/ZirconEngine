use std::fs;

use zircon_runtime::scene::DefaultLevelManager;

use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetPreviewPreset};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::workbench::project::EditorProjectDocument;

use super::support::*;

#[test]
fn editor_manager_opens_and_saves_ui_asset_editor_sessions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_session");
    let ui_asset_path = unique_temp_dir("zircon_editor_asset_session_file").join("test.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, SIMPLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, None)
        .expect("ui asset editor should open");
    let reflection = manager
        .ui_asset_editor_reflection(&instance_id)
        .expect("ui asset editor reflection");
    assert_eq!(reflection.route.asset_id, ui_asset_path.to_string_lossy());
    assert_eq!(reflection.display_name, "Test UI Asset");
    assert!(reflection.preview_available);
    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("ui asset editor pane presentation");
    assert_eq!(pane.asset_id, ui_asset_path.to_string_lossy());
    assert_eq!(pane.mode, "Design");
    assert!(pane.preview_available);
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains("root [VerticalBox]")));

    let edited = SIMPLE_UI_LAYOUT_ASSET.replace("Ready", "Edited");
    manager
        .update_ui_asset_editor_source(&instance_id, edited.clone())
        .expect("source update");
    assert!(
        manager
            .ui_asset_editor_reflection(&instance_id)
            .expect("updated reflection")
            .source_dirty
    );

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    assert!(saved.contains("Edited"));
    assert!(fs::read_to_string(&ui_asset_path)
        .expect("saved ui asset file")
        .contains("Edited"));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_preview_preset_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_preview_presets");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_preview_presets_file").join("layout.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, SIMPLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, Some(UiAssetEditorMode::Preview))
        .expect("ui asset editor should open");
    let initial = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("initial pane");
    assert_eq!(initial.preview_summary, "2 rendered nodes @ 1280x720");

    assert!(manager
        .set_ui_asset_editor_preview_preset(&instance_id, UiAssetPreviewPreset::GameHud)
        .expect("set preview preset"));
    let updated = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert_eq!(updated.preview_preset, "Game HUD");
    assert_eq!(updated.preview_summary, "2 rendered nodes @ 1920x1080");
    assert!(!updated.source_dirty);
    assert_eq!(
        manager
            .ui_asset_editor_reflection(&instance_id)
            .expect("updated reflection")
            .route
            .preview_preset,
        UiAssetPreviewPreset::GameHud
    );

    assert!(!manager
        .set_ui_asset_editor_preview_preset(&instance_id, UiAssetPreviewPreset::GameHud)
        .expect("same preset should no-op"));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_mock_preview_actions_without_dirtying_source() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_mock_preview");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_mock_preview_file").join("layout.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, MOCK_PREVIEW_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, Some(UiAssetEditorMode::Preview))
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select button");
    manager
        .select_ui_asset_editor_preview_mock_property(&instance_id, 0)
        .expect("select preview mock property");
    assert!(manager
        .set_ui_asset_editor_selected_preview_mock_value(&instance_id, "Preview Save")
        .expect("set preview mock value"));

    let updated = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert_eq!(updated.preview_mock_selected_index, 0);
    assert_eq!(updated.preview_mock_property, "text");
    assert_eq!(updated.preview_mock_kind, "Text");
    assert_eq!(updated.preview_mock_value, "Preview Save");
    assert!(updated.preview_mock_can_clear);
    assert!(!updated.source_dirty);
    assert_eq!(
        manager
            .ui_asset_editor_reflection(&instance_id)
            .expect("reflection")
            .source_dirty,
        false
    );

    assert!(manager
        .clear_ui_asset_editor_selected_preview_mock_value(&instance_id)
        .expect("clear preview mock value"));
    let cleared = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("cleared pane");
    assert_eq!(cleared.preview_mock_value, "Save");
    assert!(!cleared.preview_mock_can_clear);
    assert!(!cleared.source_dirty);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_nested_mock_preview_actions_without_dirtying_source() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_nested_mock_preview");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_nested_mock_preview_file").join("layout.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, MOCK_PREVIEW_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, Some(UiAssetEditorMode::Preview))
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select button");

    let initial = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("initial pane");
    let items_index = initial
        .preview_mock_items
        .iter()
        .position(|item| item.starts_with("items [Collection]"))
        .expect("items preview entry");
    manager
        .select_ui_asset_editor_preview_mock_property(&instance_id, items_index)
        .expect("select collection preview property");
    manager
        .select_ui_asset_editor_preview_mock_nested_entry(&instance_id, 1)
        .expect("select nested entry");
    manager
        .set_ui_asset_editor_selected_preview_mock_nested_value(&instance_id, "Ship")
        .expect("set nested value");
    manager
        .upsert_ui_asset_editor_selected_preview_mock_nested_entry(&instance_id, "2", "\"Archive\"")
        .expect("append nested entry");
    manager
        .select_ui_asset_editor_preview_mock_nested_entry(&instance_id, 0)
        .expect("reselect first nested entry");
    manager
        .delete_ui_asset_editor_selected_preview_mock_nested_entry(&instance_id)
        .expect("delete first nested entry");

    let updated = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert_eq!(updated.preview_mock_kind, "Collection");
    assert_eq!(updated.preview_mock_value, "[\"Ship\", \"Archive\"]");
    assert_eq!(
        updated.preview_mock_nested_items,
        vec![
            "[0] [Text] = Ship".to_string(),
            "[1] [Text] = Archive".to_string(),
        ]
    );
    assert_eq!(updated.preview_mock_nested_selected_index, 0);
    assert_eq!(updated.preview_mock_nested_key, "0");
    assert_eq!(updated.preview_mock_nested_value, "Ship");
    assert_eq!(
        updated.preview_state_graph_items,
        vec!["SaveButton.items = [\"Ship\", \"Archive\"]".to_string()]
    );
    assert!(!updated.source_dirty);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_resolves_ui_asset_imports_and_interactive_session_commands() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_imports");
    let project_root = unique_temp_dir("zircon_editor_asset_import_project");
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
    let style_path = project_root
        .join("assets")
        .join("ui")
        .join("styles")
        .join("theme.ui.toml");
    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.ui.toml");
    fs::create_dir_all(widget_path.parent().unwrap()).unwrap();
    fs::create_dir_all(style_path.parent().unwrap()).unwrap();
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
classes = ["primary"]
props = { text = "Press" }
"#,
    );
    write_ui_asset(
        &style_path,
        r##"
[asset]
kind = "style"
id = "ui.styles.theme"
version = 1
display_name = "Theme"

[[stylesheets]]
id = "theme"

[[stylesheets.rules]]
selector = ".primary"
set = { self.background = { color = "#224488" } }
"##,
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
styles = ["res://ui/styles/theme.ui.toml"]

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

    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("ui asset editor pane");
    assert!(pane.preview_available);
    assert!(pane
        .preview_items
        .iter()
        .any(|item| item.contains("ToolbarButton")));

    manager
        .set_ui_asset_editor_mode(&instance_id, UiAssetEditorMode::Split)
        .expect("set split mode");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select first child");

    let reflection = manager
        .ui_asset_editor_reflection(&instance_id)
        .expect("updated reflection");
    assert_eq!(reflection.route.mode, UiAssetEditorMode::Split);
    assert_eq!(
        reflection.selection.primary_node_id.as_deref(),
        Some("toolbar")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_runs_ui_asset_style_authoring_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_style_authoring");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_style_authoring_file").join("style.ui.toml");
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
        .create_ui_asset_editor_rule_from_selection(&instance_id)
        .expect("create rule from selection");
    manager
        .toggle_ui_asset_editor_pseudo_state(&instance_id, "hover")
        .expect("toggle hover preview");
    manager
        .extract_ui_asset_editor_inline_overrides_to_rule(&instance_id)
        .expect("extract inline overrides");

    let reflection = manager
        .ui_asset_editor_reflection(&instance_id)
        .expect("updated reflection");
    assert_eq!(
        reflection.style_inspector.active_pseudo_states,
        vec!["hover"]
    );
    assert!(reflection
        .style_inspector
        .matched_rules
        .iter()
        .any(|rule| rule.selector == ".primary:hover"));
    assert!(reflection
        .style_inspector
        .matched_rules
        .iter()
        .any(|rule| rule.selector == "#SaveButton"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    assert!(saved.contains("selector = \"#SaveButton\""));
    let document =
        crate::tests::support::load_test_ui_asset(&saved).expect("saved ui asset document");
    let button = document.node("button").expect("button node");
    assert!(button.style_overrides.self_values.is_empty());
    assert!(button.style_overrides.slot.is_empty());
    assert!(
        !manager
            .ui_asset_editor_reflection(&instance_id)
            .expect("saved reflection")
            .source_dirty
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_selects_ui_asset_nodes_from_source_byte_offsets() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_source_byte_offset");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_source_byte_offset_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, STYLE_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, Some(UiAssetEditorMode::Split))
        .expect("ui asset editor should open");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select button from hierarchy");
    let selected_line = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("button pane")
        .source_selected_line;
    assert!(selected_line > 0);
    let byte_offset = {
        let pane = manager
            .ui_asset_editor_pane_presentation(&instance_id)
            .expect("pane presentation");
        pane.source_cursor_byte_offset as usize
    };

    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 0)
        .expect("select root from hierarchy");
    assert!(manager
        .select_ui_asset_editor_source_byte_offset(&instance_id, byte_offset)
        .expect("select source byte offset"));

    let pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("selected pane");
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.hierarchy_selected_index, 1);
    assert_eq!(pane.source_cursor_byte_offset, byte_offset as i32);

    assert!(!manager
        .select_ui_asset_editor_source_byte_offset(&instance_id, 0)
        .expect("header offset should no-op"));
    let unchanged = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("unchanged pane");
    assert_eq!(unchanged.inspector_selected_node_id, "button");
    assert_eq!(unchanged.hierarchy_selected_index, 1);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_preview_mock_suggestion_actions_relative_to_selected_container() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_asset_mock_preview_suggestion");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_asset_mock_preview_suggestion_file").join("layout.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    write_ui_asset(&ui_asset_path, DEEP_MOCK_PREVIEW_UI_LAYOUT_ASSET);

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, Some(UiAssetEditorMode::Design))
        .expect("ui asset editor should open");

    let initial = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("initial pane");
    let status_subject_index = initial
        .preview_mock_subject_items
        .iter()
        .position(|item| item.contains("StatusLabel"))
        .expect("status subject");
    manager
        .select_ui_asset_editor_preview_mock_subject(&instance_id, status_subject_index)
        .expect("select status subject");
    let context_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("context pane")
        .preview_mock_items
        .iter()
        .position(|item| item.contains("context [Object]"))
        .expect("context property");
    manager
        .select_ui_asset_editor_preview_mock_property(&instance_id, context_index)
        .expect("select context property");

    let initial_context_pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("context pane");
    assert_eq!(
        initial_context_pane.preview_mock_suggestion_items,
        vec![
            "[0] = { label = \"Plan\" }".to_string(),
            "[1] = { label = \"Dirty\" }".to_string(),
            "[n] = { label = \"Plan\" }".to_string(),
        ]
    );

    let dialog_index = initial_context_pane
        .preview_mock_nested_items
        .iter()
        .position(|item| item.contains("dialog [Object]"))
        .expect("dialog nested entry");
    manager
        .select_ui_asset_editor_preview_mock_nested_entry(&instance_id, dialog_index)
        .expect("select dialog nested entry");
    let dialog_pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("dialog pane");
    assert_eq!(
        dialog_pane.preview_mock_suggestion_items,
        vec![
            "steps = [{ label = \"Plan\" }, { label = \"Dirty\" }]".to_string(),
            "title = \"Ready\"".to_string(),
        ]
    );

    let steps_index = dialog_pane
        .preview_mock_nested_items
        .iter()
        .position(|item| item.contains("dialog.steps [Collection]"))
        .expect("dialog.steps nested entry");
    manager
        .select_ui_asset_editor_preview_mock_nested_entry(&instance_id, steps_index)
        .expect("select dialog.steps nested entry");
    let steps_pane = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("steps pane");
    assert_eq!(
        steps_pane.preview_mock_suggestion_items,
        vec![
            "[0] = { label = \"Plan\" }".to_string(),
            "[1] = { label = \"Dirty\" }".to_string(),
            "[n] = { label = \"Plan\" }".to_string(),
        ]
    );

    assert!(manager
        .apply_ui_asset_editor_selected_preview_mock_suggestion(&instance_id, 2)
        .expect("apply preview mock suggestion"));
    let updated = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("updated pane");
    assert!(updated
        .preview_mock_nested_items
        .iter()
        .any(|item| item.contains("dialog.steps[2] [Object] = { label = \"Plan\" }")));
    assert_eq!(updated.preview_mock_nested_key, "dialog.steps[2]");
    assert_eq!(updated.preview_mock_nested_value, "{ label = \"Plan\" }");
    assert_eq!(
        updated.preview_mock_suggestion_items,
        vec!["label = \"Plan\"".to_string()]
    );
    assert!(updated
        .preview_state_graph_items
        .contains(
            &"StatusLabel.context = { dialog = { steps = [{ label = \"Plan\" }, { label = \"Dirty\" }, { label = \"Plan\" }], title = \"Ready\" } }"
                .to_string()
        ));
    assert!(!updated.source_dirty);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

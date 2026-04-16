use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_core::CoreRuntime;
use zircon_manager::{resolve_config_manager, MANAGER_MODULE_NAME};
use zircon_scene::DefaultLevelManager;
use zircon_ui::UiAssetLoader;

use crate::layout::{MainHostPageLayout, MainPageId, WorkbenchLayout};
use crate::module::module_descriptor;
use crate::project::{EditorProjectDocument, ProjectEditorWorkspace};
use crate::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};
use crate::{
    module, EditorManager, EditorSessionMode, NewProjectDraft, NewProjectTemplate,
    RecentProjectValidation, EDITOR_MANAGER_NAME,
};

fn unique_temp_path(prefix: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}.json"))
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}"))
}

const SIMPLE_UI_LAYOUT_ASSET: &str = r#"
[asset]
kind = "layout"
id = "editor.tests.asset"
version = 1
display_name = "Test UI Asset"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 8.0 } }
children = [{ child = "status" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "Status"
props = { text = "Ready" }
"#;

const STYLE_UI_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.style"
version = 1
display_name = "Styled UI Asset"

[tokens]
accent = "#4488ff"
panel_gap = 12

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
style_overrides = { self = { text = { color = "#ffffff" } } }

[[stylesheets]]
id = "local"

[[stylesheets.rules]]
selector = ".primary"
set = { self.background = { color = "#224488" } }

[[stylesheets.rules]]
selector = ".primary:hover"
set = { self.text = { color = "#ffeeaa" } }
"##;

fn env_lock() -> &'static Mutex<()> {
    crate::tests::support::env_lock()
}

fn editor_runtime_with_config_path(path: &std::path::Path) -> CoreRuntime {
    std::env::set_var("ZIRCON_EDITOR_CONFIG_PATH", path);
    let runtime = CoreRuntime::new();
    runtime
        .register_module(zircon_manager::module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_asset::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(MANAGER_MODULE_NAME).unwrap();
    runtime
        .activate_module(zircon_asset::ASSET_MODULE_NAME)
        .unwrap();
    runtime.activate_module(module::EDITOR_MODULE_NAME).unwrap();
    runtime
}

fn empty_layout_with_page(page_id: &str) -> WorkbenchLayout {
    let page_id = MainPageId::new(page_id);
    WorkbenchLayout {
        active_main_page: page_id.clone(),
        main_pages: vec![MainHostPageLayout::WorkbenchPage {
            id: page_id,
            title: "Workbench".to_string(),
            document_workspace: crate::DocumentNode::Tabs(crate::TabStackLayout {
                tabs: Vec::new(),
                active_tab: None,
            }),
        }],
        drawers: crate::ActivityDrawerSlot::ALL
            .into_iter()
            .map(|slot| {
                (
                    slot,
                    crate::ActivityDrawerLayout {
                        slot,
                        tab_stack: crate::TabStackLayout::default(),
                        active_view: None,
                        mode: crate::ActivityDrawerMode::Pinned,
                        extent: if matches!(
                            slot,
                            crate::ActivityDrawerSlot::BottomLeft
                                | crate::ActivityDrawerSlot::BottomRight
                        ) {
                            200.0
                        } else {
                            260.0
                        },
                        visible: true,
                    },
                )
            })
            .collect(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    }
}

#[test]
fn editor_manager_bootstrap_prefers_global_default_layout() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_global");
    let runtime = editor_runtime_with_config_path(&path);
    let config = resolve_config_manager(&runtime.handle()).unwrap();
    let custom_layout = empty_layout_with_page("global-layout");
    config
        .set_value(
            "editor.workbench.default_layout",
            serde_json::to_value(&custom_layout).unwrap(),
        )
        .unwrap();

    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    assert_eq!(
        manager.current_layout().active_main_page,
        custom_layout.active_main_page
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn editor_manager_bootstrap_repairs_empty_global_default_layout() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_global_empty");
    let runtime = editor_runtime_with_config_path(&path);
    let config = resolve_config_manager(&runtime.handle()).unwrap();
    let empty_layout = empty_layout_with_page("global-layout");
    config
        .set_value(
            "editor.workbench.default_layout",
            serde_json::to_value(&empty_layout).unwrap(),
        )
        .unwrap();

    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let layout = manager.current_layout();

    assert_eq!(layout.active_main_page, MainPageId::new("global-layout"));

    let left_top = layout
        .drawers
        .get(&crate::ActivityDrawerSlot::LeftTop)
        .expect("left top drawer");
    assert_eq!(
        left_top.tab_stack.tabs,
        vec![
            ViewInstanceId::new("editor.project#1"),
            ViewInstanceId::new("editor.assets#1"),
            ViewInstanceId::new("editor.hierarchy#1"),
        ]
    );
    assert_eq!(
        left_top.active_view,
        Some(ViewInstanceId::new("editor.project#1"))
    );

    let right_top = layout
        .drawers
        .get(&crate::ActivityDrawerSlot::RightTop)
        .expect("right top drawer");
    assert_eq!(
        right_top.tab_stack.tabs,
        vec![ViewInstanceId::new("editor.inspector#1")]
    );

    let bottom_left = layout
        .drawers
        .get(&crate::ActivityDrawerSlot::BottomLeft)
        .expect("bottom left drawer");
    assert_eq!(
        bottom_left.tab_stack.tabs,
        vec![ViewInstanceId::new("editor.console#1")]
    );

    let workbench_page = layout
        .main_pages
        .iter()
        .find_map(|page| match page {
            MainHostPageLayout::WorkbenchPage {
                document_workspace, ..
            } => Some(document_workspace),
            MainHostPageLayout::ExclusiveActivityWindowPage { .. } => None,
        })
        .expect("workbench page");
    let crate::DocumentNode::Tabs(document_tabs) = workbench_page else {
        panic!("expected root document tabs");
    };
    assert_eq!(
        document_tabs.tabs,
        vec![
            ViewInstanceId::new("editor.scene#1"),
            ViewInstanceId::new("editor.game#1"),
        ]
    );
    assert_eq!(
        document_tabs.active_tab,
        Some(ViewInstanceId::new("editor.scene#1"))
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn save_and_load_preset_roundtrip_through_manager_commands() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_presets");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    manager
        .apply_layout_command(crate::LayoutCommand::ActivateMainPage {
            page_id: MainPageId::new("preset-page"),
        })
        .unwrap();
    manager
        .apply_layout_command(crate::LayoutCommand::SavePreset {
            name: "authoring".to_string(),
        })
        .unwrap();
    manager
        .apply_layout_command(crate::LayoutCommand::ResetToDefault)
        .unwrap();
    manager
        .apply_layout_command(crate::LayoutCommand::LoadPreset {
            name: "authoring".to_string(),
        })
        .unwrap();

    assert_eq!(
        manager.current_layout().active_main_page,
        MainPageId::new("preset-page")
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn save_and_load_preset_roundtrip_through_project_asset_files() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_project_presets");
    let project_root = unique_temp_dir("zircon_editor_project_presets");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();
    manager.open_project(&project_root).unwrap();

    manager
        .apply_layout_command(crate::LayoutCommand::ActivateMainPage {
            page_id: MainPageId::new("preset-page"),
        })
        .unwrap();
    manager
        .apply_layout_command(crate::LayoutCommand::SavePreset {
            name: "rider".to_string(),
        })
        .unwrap();

    let preset_asset = project_root
        .join("assets")
        .join("editor")
        .join("layout-presets")
        .join("rider.workbench-layout.json");
    assert!(
        preset_asset.exists(),
        "expected preset asset at {:?}",
        preset_asset
    );

    manager
        .apply_layout_command(crate::LayoutCommand::ResetToDefault)
        .unwrap();
    manager
        .apply_layout_command(crate::LayoutCommand::LoadPreset {
            name: "rider".to_string(),
        })
        .unwrap();

    assert_eq!(
        manager.current_layout().active_main_page,
        MainPageId::new("preset-page")
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn applying_project_workspace_restores_single_instance_registry_state() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let restored_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.hierarchy#restored"),
        descriptor_id: ViewDescriptorId::new("editor.hierarchy"),
        title: "Hierarchy".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::Drawer(crate::ActivityDrawerSlot::LeftTop),
    };
    let workspace = ProjectEditorWorkspace {
        layout_version: 1,
        workbench: WorkbenchLayout {
            active_main_page: MainPageId::workbench(),
            main_pages: vec![MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: "Workbench".to_string(),
                document_workspace: crate::DocumentNode::default(),
            }],
            drawers: BTreeMap::from([(
                crate::ActivityDrawerSlot::LeftTop,
                crate::ActivityDrawerLayout {
                    slot: crate::ActivityDrawerSlot::LeftTop,
                    tab_stack: crate::TabStackLayout {
                        tabs: vec![restored_instance.instance_id.clone()],
                        active_tab: Some(restored_instance.instance_id.clone()),
                    },
                    active_view: Some(restored_instance.instance_id.clone()),
                    mode: crate::ActivityDrawerMode::Pinned,
                    extent: 260.0,
                    visible: true,
                },
            )]),
            floating_windows: Vec::new(),
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        },
        open_view_instances: vec![restored_instance.clone()],
        active_center_tab: None,
        active_drawers: vec![crate::ActivityDrawerSlot::LeftTop],
    };

    manager.apply_project_workspace(Some(workspace)).unwrap();
    let reopened = manager
        .open_view(ViewDescriptorId::new("editor.hierarchy"), None)
        .unwrap();

    assert_eq!(reopened, restored_instance.instance_id);

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn scene_and_game_tabs_are_not_closeable() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_non_closeable_docs");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    assert!(!manager
        .close_view(&ViewInstanceId::new("editor.scene#1"))
        .unwrap());
    assert!(!manager
        .close_view(&ViewInstanceId::new("editor.game#1"))
        .unwrap());
    assert!(manager
        .current_view_instances()
        .iter()
        .any(|instance| instance.instance_id.0 == "editor.scene#1"));
    assert!(manager
        .current_view_instances()
        .iter()
        .any(|instance| instance.instance_id.0 == "editor.game#1"));

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn startup_session_defaults_to_welcome_without_recent_project() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_startup_welcome");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let session = manager.resolve_startup_session().unwrap();

    assert_eq!(session.mode, EditorSessionMode::Welcome);
    assert!(session.project.is_none());
    assert!(session.recent_projects.is_empty());
    assert_eq!(session.draft.project_name, "ZirconProject");
    assert_eq!(session.draft.template, NewProjectTemplate::RenderableEmpty);

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn create_project_and_open_persists_recent_project_and_returns_project_session() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_startup_recent");
    let location = unique_temp_dir("zircon_editor_welcome_recent");
    fs::create_dir_all(&location).unwrap();
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let draft = NewProjectDraft {
        project_name: "RecentProject".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };

    let opened = manager.create_project_and_open(draft).unwrap();
    let recent = manager.recent_projects_snapshot().unwrap();
    let reopened = manager.resolve_startup_session().unwrap();

    assert_eq!(opened.mode, EditorSessionMode::Project);
    assert!(opened.project.is_some());
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].display_name, "RecentProject");
    assert_eq!(recent[0].validation, RecentProjectValidation::Valid);
    assert_eq!(reopened.mode, EditorSessionMode::Project);
    assert!(reopened.project.is_some());

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(location);
}

#[test]
fn editor_manager_opens_and_saves_ui_asset_editor_sessions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_session");
    let ui_asset_path = unique_temp_dir("zircon_editor_ui_asset_session_file").join("test.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, SIMPLE_UI_LAYOUT_ASSET).unwrap();

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

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_resolves_ui_asset_imports_and_interactive_session_commands() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_imports");
    let project_root = unique_temp_dir("zircon_editor_ui_asset_import_project");
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
    fs::write(
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
    )
    .unwrap();
    fs::write(
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
    )
    .unwrap();
    fs::write(
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
    )
    .unwrap();

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
        .set_ui_asset_editor_mode(&instance_id, crate::UiAssetEditorMode::Split)
        .expect("set split mode");
    manager
        .select_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("select first child");

    let reflection = manager
        .ui_asset_editor_reflection(&instance_id)
        .expect("updated reflection");
    assert_eq!(reflection.route.mode, crate::UiAssetEditorMode::Split);
    assert_eq!(
        reflection.selection.primary_node_id.as_deref(),
        Some("toolbar")
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_runs_ui_asset_style_authoring_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_style_authoring");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_style_authoring_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, STYLE_UI_LAYOUT_ASSET).unwrap();

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
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved ui asset document");
    let button = document.nodes.get("button").expect("button node");
    assert!(button.style_overrides.self_values.is_empty());
    assert!(button.style_overrides.slot.is_empty());
    assert!(
        !manager
            .ui_asset_editor_reflection(&instance_id)
            .expect("saved reflection")
            .source_dirty
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_style_class_editing_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_style_classes");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_style_classes_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, STYLE_UI_LAYOUT_ASSET).unwrap();

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
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved ui asset document");
    let button = document.nodes.get("button").expect("button node");
    assert_eq!(button.classes, vec!["toolbar".to_string()]);

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_style_rule_editing_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_style_rules");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_style_rules_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, STYLE_UI_LAYOUT_ASSET).unwrap();

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
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved ui asset document");
    let selectors = document.stylesheets[0]
        .rules
        .iter()
        .map(|rule| rule.selector.clone())
        .collect::<Vec<_>>();
    assert_eq!(selectors, vec![".primary".to_string()]);

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_style_rule_declaration_editing_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_style_rule_declarations");
    let ui_asset_path = unique_temp_dir("zircon_editor_ui_asset_style_rule_declarations_file")
        .join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, STYLE_UI_LAYOUT_ASSET).unwrap();

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
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved ui asset document");
    let rule = &document.stylesheets[0].rules[0];
    assert!(rule.set.self_values.is_empty());
    assert!(rule.set.slot.is_empty());

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_projects_matched_style_rule_summaries_into_stylesheet_items() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_matched_rule_details");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_matched_rule_details_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, STYLE_UI_LAYOUT_ASSET).unwrap();

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

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_projects_selected_matched_style_rule_details() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_matched_rule_details");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_matched_rule_details_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, STYLE_UI_LAYOUT_ASSET).unwrap();

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

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_widget_inspector_editing_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_widget_inspector");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_widget_inspector_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, STYLE_UI_LAYOUT_ASSET).unwrap();

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
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved ui asset document");
    let button = document.nodes.get("button").expect("button node");
    assert_eq!(button.control_id.as_deref(), Some("ConfirmButton"));
    assert_eq!(
        button.props.get("text").and_then(toml::Value::as_str),
        Some("Confirm")
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_slot_inspector_editing_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_slot_inspector");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_slot_inspector_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, STYLE_UI_LAYOUT_ASSET).unwrap();

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
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved ui asset document");
    let child_mount = document.nodes["root"]
        .children
        .iter()
        .find(|child_mount| child_mount.child == "button")
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

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_layout_inspector_editing_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_layout_inspector");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_layout_inspector_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, STYLE_UI_LAYOUT_ASSET).unwrap();

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
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved ui asset document");
    let button = document.nodes.get("button").expect("button node");
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

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_binding_inspector_editing_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_binding_inspector");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_binding_inspector_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, STYLE_UI_LAYOUT_ASSET).unwrap();

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
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved ui asset document");
    let button = document.nodes.get("button").expect("button node");
    assert_eq!(button.bindings.len(), 1);
    assert_eq!(button.bindings[0].id, "SaveButton/onHover");
    assert_eq!(button.bindings[0].event.to_string(), "onHover");
    assert_eq!(
        button.bindings[0].route.as_deref(),
        Some("MenuAction.HighlightSave")
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_style_token_editing_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_style_tokens");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_style_tokens_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, STYLE_UI_LAYOUT_ASSET).unwrap();

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
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved ui asset document");
    assert!(!document.tokens.contains_key("accent"));
    assert!(!document.tokens.contains_key("accent_primary"));
    assert_eq!(
        document
            .tokens
            .get("surface_fill")
            .and_then(toml::Value::as_str),
        Some("#223344")
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

fn slot_value<'a>(
    slot: &'a std::collections::BTreeMap<String, toml::Value>,
    path: &[&str],
) -> Option<&'a toml::Value> {
    let (first, rest) = path.split_first()?;
    let value = slot.get(*first)?;
    if rest.is_empty() {
        return Some(value);
    }
    let toml::Value::Table(table) = value else {
        return None;
    };
    slot_table_value(table, rest)
}

fn layout_value<'a>(
    layout: Option<&'a std::collections::BTreeMap<String, toml::Value>>,
    path: &[&str],
) -> Option<&'a toml::Value> {
    let layout = layout?;
    slot_value(layout, path)
}

fn slot_table_value<'a>(
    table: &'a toml::map::Map<String, toml::Value>,
    path: &[&str],
) -> Option<&'a toml::Value> {
    let (first, rest) = path.split_first()?;
    let value = table.get(*first)?;
    if rest.is_empty() {
        return Some(value);
    }
    let toml::Value::Table(child) = value else {
        return None;
    };
    slot_table_value(child, rest)
}

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_asset::UiWidgetAsset;
use zircon_core::CoreRuntime;
use zircon_foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};
use zircon_manager::resolve_config_manager;
use zircon_scene::DefaultLevelManager;
use zircon_ui::UiAssetLoader;

use crate::layout::{MainHostPageLayout, MainPageId, WorkbenchLayout};
use crate::module::module_descriptor;
use crate::project::{EditorProjectDocument, ProjectEditorWorkspace};
use crate::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};
use crate::{
    module, EditorManager, EditorSessionMode, NewProjectDraft, NewProjectTemplate,
    RecentProjectValidation, UiAssetPreviewPreset, EDITOR_MANAGER_NAME,
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

fn byte_offset_for_line(source: &str, line: usize) -> usize {
    if line <= 1 {
        return 0;
    }
    let mut current_line = 1usize;
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            current_line += 1;
            if current_line == line {
                return index + 1;
            }
        }
    }
    source.len()
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

const MOCK_PREVIEW_UI_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.mock_preview"
version = 1
display_name = "Mock Preview UI Asset"

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
props = { text = "Save", checked = false, mode = "Full", icon = "asset://ui/icons/save.png" }
"##;

const TREE_REPARENT_UI_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.tree_reparent"
version = 1
display_name = "Tree Reparent UI Asset"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "group_a" }, { child = "loose" }, { child = "group_b" }]

[nodes.group_a]
kind = "native"
type = "VerticalBox"
control_id = "GroupA"
children = [{ child = "nested_a" }]

[nodes.nested_a]
kind = "native"
type = "Label"
control_id = "NestedA"
props = { text = "Nested A" }

[nodes.loose]
kind = "native"
type = "Button"
control_id = "LooseButton"
props = { text = "Loose" }

[nodes.group_b]
kind = "native"
type = "VerticalBox"
control_id = "GroupB"
children = [{ child = "nested_b" }]

[nodes.nested_b]
kind = "native"
type = "Label"
control_id = "NestedB"
props = { text = "Nested B" }
"##;

const SEMANTIC_UI_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.semantic"
version = 1
display_name = "Semantic UI Asset"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Overlay"
control_id = "Root"
children = [{ child = "scroll_panel", slot = { layout = { anchor = { x = 1.0, y = 0.0 }, pivot = { x = 1.0, y = 0.0 }, position = { x = -16.0, y = 12.0 }, z_index = 4 } } }]

[nodes.scroll_panel]
kind = "native"
type = "ScrollableBox"
control_id = "ScrollPanel"
layout = { container = { kind = "ScrollableBox", axis = "Vertical", gap = 6, scrollbar_visibility = "Always", virtualization = { item_extent = 28, overscan = 2 } }, clip = true }
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "ActionButton"
props = { text = "Run" }
"##;

const STRUCTURED_BINDING_UI_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.structured_binding"
version = 1
display_name = "Structured Binding UI Asset"

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
props = { text = "Save" }
bindings = [{ id = "SaveButton/onClick", event = "Click", route = "MenuAction.SaveProject", action = { route = "MenuAction.SaveProject", payload = { confirm = true, mode = "full" } } }]
"##;

fn env_lock() -> &'static Mutex<()> {
    crate::tests::support::env_lock()
}

fn editor_runtime_with_config_path(path: &std::path::Path) -> CoreRuntime {
    std::env::set_var("ZIRCON_EDITOR_CONFIG_PATH", path);
    let runtime = CoreRuntime::new();
    runtime
        .register_module(foundation_module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_asset::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
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
fn editor_manager_runs_ui_asset_preview_preset_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_preview_presets");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_preview_presets_file").join("layout.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, SIMPLE_UI_LAYOUT_ASSET).unwrap();

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, Some(crate::UiAssetEditorMode::Preview))
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

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_mock_preview_actions_without_dirtying_source() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_mock_preview");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_mock_preview_file").join("layout.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, MOCK_PREVIEW_UI_LAYOUT_ASSET).unwrap();

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, Some(crate::UiAssetEditorMode::Preview))
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
fn editor_manager_selects_ui_asset_nodes_from_source_byte_offsets() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_source_byte_offset");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_source_byte_offset_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, STYLE_UI_LAYOUT_ASSET).unwrap();

    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_ui_asset_editor(&ui_asset_path, Some(crate::UiAssetEditorMode::Split))
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
        byte_offset_for_line(&pane.source_text, (selected_line + 1) as usize)
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

    assert!(!manager
        .select_ui_asset_editor_source_byte_offset(&instance_id, 0)
        .expect("header offset should no-op"));
    let unchanged = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("unchanged pane");
    assert_eq!(unchanged.inspector_selected_node_id, "button");
    assert_eq!(unchanged.hierarchy_selected_index, 1);

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
fn editor_manager_runs_ui_asset_parent_specific_semantic_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_semantic_inspector");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_semantic_inspector_file").join("semantic.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, SEMANTIC_UI_LAYOUT_ASSET).unwrap();

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
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved ui asset document");
    let child_mount = document.nodes["root"]
        .children
        .iter()
        .find(|child_mount| child_mount.child == "scroll_panel")
        .expect("scroll panel child mount");
    assert_eq!(
        slot_value(&child_mount.slot, &["layout", "anchor", "x"]).and_then(toml::Value::as_float),
        Some(0.5)
    );
    let scroll_panel = document.nodes.get("scroll_panel").expect("scroll panel");
    assert_eq!(
        layout_value(scroll_panel.layout.as_ref(), &["container", "axis"])
            .and_then(toml::Value::as_str),
        Some("Horizontal")
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
fn editor_manager_runs_ui_asset_structured_binding_inspector_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_structured_binding_inspector");
    let ui_asset_path = unique_temp_dir("zircon_editor_ui_asset_structured_binding_inspector_file")
        .join("structured-binding.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, STRUCTURED_BINDING_UI_LAYOUT_ASSET).unwrap();

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
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved ui asset document");
    let button = document.nodes.get("button").expect("button node");
    assert_eq!(button.bindings[0].event.to_string(), "onDoubleClick");
    assert!(button.bindings[0].route.is_none());
    let action = button.bindings[0].action.as_ref().expect("binding action");
    assert_eq!(action.action.as_deref(), Some("EditorActions.SaveProject"));
    assert_eq!(
        action.payload.get("mode").and_then(toml::Value::as_str),
        Some("compact")
    );
    assert!(action.payload.get("channel").is_none());

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_runs_ui_asset_palette_and_tree_authoring_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_tree_authoring");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_tree_authoring_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, STYLE_UI_LAYOUT_ASSET).unwrap();

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
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved ui asset document");
    assert!(document.nodes.contains_key("button_2"));
    assert_eq!(
        document.nodes.get("root").map(|node| node
            .children
            .iter()
            .map(|child| child.child.clone())
            .collect::<Vec<_>>()),
        Some(vec!["button".to_string(), "button_2".to_string()])
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_restores_ui_asset_tree_selection_across_undo_and_redo() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_tree_selection_undo");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_tree_selection_undo_file").join("style.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, STYLE_UI_LAYOUT_ASSET).unwrap();

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

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_opens_selected_ui_asset_reference_in_new_editor_instance() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_open_reference");
    let project_root = unique_temp_dir("zircon_editor_ui_asset_open_reference_project");
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
props = { text = "Press" }
"#,
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
    assert_eq!(reflection.route.asset_kind, zircon_ui::UiAssetKind::Widget);

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_activates_selected_ui_asset_reference_from_hierarchy() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_activate_reference");
    let project_root = unique_temp_dir("zircon_editor_ui_asset_activate_reference_project");
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
props = { text = "Press" }
"#,
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

    let opened = manager
        .activate_ui_asset_editor_hierarchy_index(&instance_id, 1)
        .expect("activate hierarchy item")
        .expect("reference view instance");

    let reflection = manager
        .ui_asset_editor_reflection(&opened)
        .expect("reference reflection");
    assert_eq!(reflection.route.asset_id, "res://ui/widgets/button.ui.toml");
    assert_eq!(reflection.display_name, "Toolbar Button");
    assert_eq!(reflection.route.asset_kind, zircon_ui::UiAssetKind::Widget);

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_activates_selected_ui_asset_reference_from_preview() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_activate_preview_reference");
    let project_root = unique_temp_dir("zircon_editor_ui_asset_activate_preview_reference_project");
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
props = { text = "Press" }
"#,
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
    assert_eq!(reflection.route.asset_kind, zircon_ui::UiAssetKind::Widget);

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_runs_ui_asset_reparent_actions() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_tree_reparent");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_tree_reparent_file").join("tree.ui.toml");
    fs::create_dir_all(ui_asset_path.parent().unwrap()).unwrap();
    fs::write(&ui_asset_path, TREE_REPARENT_UI_LAYOUT_ASSET).unwrap();

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
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved ui asset document");
    assert_eq!(
        document.nodes.get("group_b").map(|node| node
            .children
            .iter()
            .map(|child| child.child.clone())
            .collect::<Vec<_>>()),
        Some(vec!["loose".to_string(), "nested_b".to_string()])
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_converts_selected_ui_asset_node_to_reference_from_palette_selection() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_convert_reference");
    let project_root = unique_temp_dir("zircon_editor_ui_asset_convert_reference_project");
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
    fs::write(
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
    )
    .unwrap();
    fs::write(
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
    )
    .unwrap();

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
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved ui asset document");
    let button = document.nodes.get("button").expect("button node");
    assert_eq!(button.kind, zircon_ui::UiNodeDefinitionKind::Reference);
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

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_extracts_selected_ui_asset_node_to_local_component() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_extract_component");
    let ui_asset_path =
        unique_temp_dir("zircon_editor_ui_asset_extract_component_file").join("style.ui.toml");
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
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved ui asset document");
    let component = document
        .components
        .get("SaveButton")
        .expect("new local component");
    assert_eq!(
        document
            .nodes
            .get("button")
            .and_then(|node| node.component.as_deref()),
        Some("SaveButton")
    );
    assert_eq!(
        document
            .nodes
            .get(&component.root)
            .and_then(|node| node.widget_type.as_deref()),
        Some("Button")
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(ui_asset_path.parent().unwrap());
}

#[test]
fn editor_manager_promotes_selected_ui_asset_component_to_external_widget_asset() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_promote_widget");
    let project_root = unique_temp_dir("zircon_editor_ui_asset_promote_widget_project");
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
    fs::write(&layout_path, STYLE_UI_LAYOUT_ASSET).unwrap();

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
            .map(|root| root.node.as_str()),
        Some("savebutton_root")
    );
    assert!(widget_asset.document.components.contains_key("SaveButton"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save ui asset editor");
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved ui asset document");
    let button = document.nodes.get("button").expect("button node");
    assert_eq!(button.kind, zircon_ui::UiNodeDefinitionKind::Reference);
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

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_uses_custom_promote_widget_draft_values() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_ui_asset_promote_widget_custom");
    let project_root = unique_temp_dir("zircon_editor_ui_asset_promote_widget_custom_project");
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
    fs::write(&layout_path, STYLE_UI_LAYOUT_ASSET).unwrap();

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
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved ui asset document");
    let button = document.nodes.get("button").expect("button node");
    assert_eq!(
        button.component_ref.as_deref(),
        Some("res://ui/widgets/custom/editor_save.ui.toml#EditorSaveButton")
    );
    assert!(document.imports.widgets.iter().any(|reference| {
        reference == "res://ui/widgets/custom/editor_save.ui.toml#EditorSaveButton"
    }));

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
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

#[test]
fn editor_manager_ui_asset_sessions_are_split_by_host_orchestration_behaviors() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("host")
        .join("manager")
        .join("ui_asset_sessions");

    for relative in [
        "mod.rs",
        "open.rs",
        "save.rs",
        "lifecycle.rs",
        "sync.rs",
        "imports.rs",
        "hydration.rs",
        "preview_refresh.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected host ui asset session module {relative} under {:?}",
            root
        );
    }
}

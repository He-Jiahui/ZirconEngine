use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use slint::{Model, PhysicalSize};
use zircon_runtime_interface::resource::{ResourceKind, ResourceState};

use crate::ui::animation_editor::AnimationEditorPanePresentation;
use crate::ui::asset_editor::UiAssetEditorPanePresentation;
use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, ModulePluginsPaneViewData,
};
use crate::ui::slint_host::floating_window_projection::build_floating_window_projection_bundle;
use crate::ui::slint_host::{
    apply_presentation, FrameRect, HostChromeControlFrameData, HostClosePromptData,
    HostMenuChromeData, HostMenuChromeItemData, HostMenuChromeMenuData, HostMenuStateData,
    HostWindowLayoutData, TemplateNodeFrameData, TemplatePaneNodeData, UiHostContext, UiHostWindow,
};
use crate::ui::workbench::autolayout::{
    compute_workbench_shell_geometry, ShellSizePx, WorkbenchChromeMetrics,
};
use crate::ui::workbench::fixture::{default_preview_fixture, PreviewFixture};
use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, MainHostPageLayout, MainPageId, WorkbenchLayout,
};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::{
    AssetFolderSnapshot, AssetItemSnapshot, AssetSelectionSnapshot, AssetWorkspaceSnapshot,
    EditorChromeSnapshot,
};
use crate::ui::workbench::startup::{
    EditorSessionMode, NewProjectFormSnapshot, RecentProjectItemSnapshot, RecentProjectValidation,
    WelcomePaneSnapshot, WELCOME_DESCRIPTOR_ID, WELCOME_INSTANCE_ID, WELCOME_PAGE_ID,
};
use crate::ui::workbench::view::{
    PreferredHost, ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId,
    ViewKind,
};

const SCROLLED_WINDOW_POPUP_SCREENSHOT: &str =
    "editor-window-20260429-window-popup-scrolled-900x620.png";
const CLOSE_PROMPT_SCREENSHOT: &str = "editor-window-20260507-close-prompt-900x620.png";
const NESTED_MENU_POPUP_SCREENSHOT: &str = "editor-window-20260507-nested-menu-popup-900x620.png";
const M3_WELCOME_INPUT_SCREENSHOT: &str = "editor-window-m3-welcome-input-900x620.png";
const M3_WORKBENCH_SCREENSHOT: &str = "editor-window-m3-workbench-900x620.png";
const M3_ASSET_BROWSER_SCREENSHOT: &str = "editor-window-m3-asset-browser-900x620.png";
const M3_DRAWER_SCREENSHOT: &str = "editor-window-m3-assets-drawer-900x620.png";
const M3_MENU_POPUP_SCREENSHOT: &str = "editor-window-m3-menu-popup-svg-icons-900x620.png";
const M3_DRAG_AFTER_RELEASE_SCREENSHOT: &str = "editor-window-m3-drag-after-release-900x620.png";
const M3_SVG_ICON_SMALL_SCREENSHOT: &str = "editor-window-m3-svg-icon-scale-small-640x420.png";
const M3_SVG_ICON_LARGE_SCREENSHOT: &str = "editor-window-m3-svg-icon-scale-large-1260x780.png";

#[test]
#[ignore = "writes visual screenshot artifact for manual popup closeout"]
fn capture_scrolled_window_popup_visual_artifact() {
    std::env::set_var("SLINT_BACKEND", "software");

    let shell_size = ShellSizePx::new(900.0, 620.0);
    let metrics = WorkbenchChromeMetrics::default();
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        shell_size,
        &metrics,
        None,
    );
    let floating_window_projection_bundle =
        build_floating_window_projection_bundle(&model, None, &metrics, &[]);
    let ui_asset_panes: BTreeMap<String, UiAssetEditorPanePresentation> = BTreeMap::new();
    let animation_panes: BTreeMap<String, AnimationEditorPanePresentation> = BTreeMap::new();
    let module_plugins = ModulePluginsPaneViewData::default();
    let build_export = BuildExportPaneViewData::default();
    let preset_names = (0..24)
        .map(|index| format!("Preset {index:02}"))
        .collect::<Vec<_>>();
    let ui = UiHostWindow::new().expect("workbench shell should instantiate for screenshot");

    ui.show()
        .expect("workbench shell should show for screenshot capture");
    ui.window().set_size(PhysicalSize::new(900, 620));
    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &preset_names,
        Some("Preset 03"),
        &ui_asset_panes,
        &animation_panes,
        None,
        &module_plugins,
        &build_export,
        None,
        &floating_window_projection_bundle,
        None,
    );
    ui.global::<UiHostContext>()
        .set_menu_state(HostMenuStateData {
            open_menu_index: 5,
            hovered_menu_index: -1,
            hovered_menu_item_index: 17,
            window_menu_scroll_px: 360.0,
            window_menu_popup_height_px: 192.0,
            ..HostMenuStateData::default()
        });

    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("software renderer should capture the scrolled Window popup");
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should live under the repository root")
        .join("target")
        .join("visual-layout");
    std::fs::create_dir_all(&output_dir).expect("visual-layout output directory should exist");
    let output_path = output_dir.join(SCROLLED_WINDOW_POPUP_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        snapshot.as_bytes(),
        snapshot.width(),
        snapshot.height(),
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("scrolled Window popup screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

#[test]
#[ignore = "writes visual screenshot artifact for manual close-prompt closeout"]
fn capture_close_prompt_visual_artifact() {
    std::env::set_var("SLINT_BACKEND", "software");

    let shell_size = ShellSizePx::new(900.0, 620.0);
    let metrics = WorkbenchChromeMetrics::default();
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        shell_size,
        &metrics,
        None,
    );
    let floating_window_projection_bundle =
        build_floating_window_projection_bundle(&model, None, &metrics, &[]);
    let ui_asset_panes: BTreeMap<String, UiAssetEditorPanePresentation> = BTreeMap::new();
    let animation_panes: BTreeMap<String, AnimationEditorPanePresentation> = BTreeMap::new();
    let module_plugins = ModulePluginsPaneViewData::default();
    let build_export = BuildExportPaneViewData::default();
    let ui = UiHostWindow::new().expect("workbench shell should instantiate for screenshot");

    ui.show()
        .expect("workbench shell should show for screenshot capture");
    ui.window().set_size(PhysicalSize::new(900, 620));
    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        &animation_panes,
        None,
        &module_plugins,
        &build_export,
        None,
        &floating_window_projection_bundle,
        None,
    );
    ui.set_close_prompt(HostClosePromptData {
        visible: true,
        target_window_id: "drawer-window:inspector".into(),
        title: "Save changes before closing?".into(),
        message: "Inspector Drawer has unsaved changes.".into(),
        details: "Inspector Drawer\nMaterial UI Asset".into(),
        can_save: true,
        overlay_frame: frame(0.0, 0.0, 900.0, 620.0),
        dialog_frame: frame(248.0, 190.0, 404.0, 230.0),
        save_button_frame: frame(278.0, 364.0, 102.0, 30.0),
        discard_button_frame: frame(390.0, 364.0, 116.0, 30.0),
        cancel_button_frame: frame(516.0, 364.0, 104.0, 30.0),
    });

    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("software renderer should capture the close prompt");
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should live under the repository root")
        .join("target")
        .join("visual-layout");
    std::fs::create_dir_all(&output_dir).expect("visual-layout output directory should exist");
    let output_path = output_dir.join(CLOSE_PROMPT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        snapshot.as_bytes(),
        snapshot.width(),
        snapshot.height(),
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("close prompt screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

#[test]
#[ignore = "writes visual screenshot artifact for nested menu popup closeout"]
fn capture_nested_menu_popup_visual_artifact() {
    std::env::set_var("SLINT_BACKEND", "software");

    let ui = workbench_fixture_window(900, 620);
    let mut presentation = ui.get_host_presentation();
    let layout = host_window_layout_for_visual_artifact(900.0, 620.0);
    presentation.host_layout = layout.clone();
    presentation.host_scene_data.layout = layout;
    presentation.host_scene_data.menu_chrome = nested_menu_chrome_for_visual_artifact();
    presentation.menu_state = HostMenuStateData {
        open_menu_index: 0,
        hovered_menu_index: 0,
        hovered_menu_item_index: 1,
        hovered_menu_item_path: vec![0, 0],
        open_submenu_path: vec![0],
        ..HostMenuStateData::default()
    };
    let menu_state = presentation.menu_state.clone();
    ui.set_host_presentation(presentation);
    ui.global::<UiHostContext>().set_menu_state(menu_state);

    save_window_snapshot(&ui, NESTED_MENU_POPUP_SCREENSHOT);
}

#[test]
#[ignore = "writes M3 GUI screenshot artifacts for editor host cutover acceptance"]
fn capture_m3_gui_acceptance_visual_artifacts() {
    std::env::set_var("SLINT_BACKEND", "software");

    let workbench = workbench_fixture_window(900, 620);
    save_window_snapshot(&workbench, M3_WORKBENCH_SCREENSHOT);

    let welcome = welcome_input_window(900, 620);
    save_window_snapshot(&welcome, M3_WELCOME_INPUT_SCREENSHOT);

    let asset_browser = asset_browser_window(900, 620);
    save_window_snapshot(&asset_browser, M3_ASSET_BROWSER_SCREENSHOT);

    let drawer = assets_drawer_window(900, 620);
    save_window_snapshot(&drawer, M3_DRAWER_SCREENSHOT);

    let preset_names = window_menu_preset_names();
    let menu_popup =
        workbench_fixture_window_with_presets(900, 620, &preset_names, Some("Preset 03"));
    menu_popup
        .global::<UiHostContext>()
        .set_menu_state(HostMenuStateData {
            open_menu_index: 5,
            hovered_menu_index: -1,
            hovered_menu_item_index: 17,
            window_menu_scroll_px: 360.0,
            window_menu_popup_height_px: 192.0,
            ..HostMenuStateData::default()
        });
    save_window_snapshot(&menu_popup, M3_MENU_POPUP_SCREENSHOT);

    let drag_after_release = workbench_fixture_window(900, 620);
    release_first_document_tab_drag(&drag_after_release);
    save_window_snapshot(&drag_after_release, M3_DRAG_AFTER_RELEASE_SCREENSHOT);

    let small = workbench_fixture_window(640, 420);
    save_window_snapshot(&small, M3_SVG_ICON_SMALL_SCREENSHOT);
    let large = workbench_fixture_window(1260, 780);
    save_window_snapshot(&large, M3_SVG_ICON_LARGE_SCREENSHOT);
}

fn workbench_fixture_window(width: u32, height: u32) -> UiHostWindow {
    workbench_fixture_window_with_presets(width, height, &[], None)
}

fn workbench_fixture_window_with_presets(
    width: u32,
    height: u32,
    preset_names: &[String],
    active_preset_name: Option<&str>,
) -> UiHostWindow {
    let fixture = default_preview_fixture();
    presented_window_from_fixture(&fixture, width, height, preset_names, active_preset_name)
}

fn assets_drawer_window(width: u32, height: u32) -> UiHostWindow {
    let mut fixture = default_preview_fixture();
    let active = ViewInstanceId::new("editor.assets#1");
    if let Some(drawer) = fixture.layout.drawers.get_mut(&ActivityDrawerSlot::LeftTop) {
        if !drawer.tab_stack.tabs.contains(&active) {
            drawer.tab_stack.tabs.push(active.clone());
        }
        drawer.tab_stack.active_tab = Some(active.clone());
        drawer.active_view = Some(active);
        drawer.mode = ActivityDrawerMode::Pinned;
        drawer.visible = true;
    }
    presented_window_from_fixture(&fixture, width, height, &[], None)
}

fn welcome_input_window(width: u32, height: u32) -> UiHostWindow {
    let mut fixture = default_preview_fixture();
    let welcome_page_id = MainPageId::new(WELCOME_PAGE_ID);
    let welcome_instance_id = ViewInstanceId::new(WELCOME_INSTANCE_ID);

    fixture
        .descriptors
        .retain(|descriptor| descriptor.descriptor_id.0 != WELCOME_DESCRIPTOR_ID);
    fixture.descriptors.push(
        ViewDescriptor::new(
            ViewDescriptorId::new(WELCOME_DESCRIPTOR_ID),
            ViewKind::ActivityWindow,
            "Welcome",
        )
        .with_preferred_host(PreferredHost::ExclusiveMainPage)
        .with_icon_key("sparkles-outline"),
    );
    fixture
        .instances
        .retain(|instance| instance.instance_id != welcome_instance_id);
    fixture.instances.push(ViewInstance {
        instance_id: welcome_instance_id.clone(),
        descriptor_id: ViewDescriptorId::new(WELCOME_DESCRIPTOR_ID),
        title: "Welcome".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::ExclusivePage(welcome_page_id.clone()),
    });
    fixture.layout = WorkbenchLayout {
        active_main_page: welcome_page_id.clone(),
        main_pages: vec![MainHostPageLayout::ExclusiveActivityWindowPage {
            id: welcome_page_id,
            title: "Welcome".to_string(),
            window_instance: welcome_instance_id,
        }],
        drawers: BTreeMap::new(),
        activity_windows: BTreeMap::new(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };

    let mut data = fixture.editor.clone().into_snapshot();
    data.status_line = "Welcome input commit preview: B".to_string();
    data.project_path.clear();
    data.session_mode = EditorSessionMode::Welcome;
    data.project_open = false;
    data.can_undo = false;
    data.can_redo = false;
    data.welcome = WelcomePaneSnapshot {
        title: "Zircon Editor".to_string(),
        subtitle: "Create or open a project".to_string(),
        status_message: "Material text field accepted B through the editor binding path."
            .to_string(),
        browse_supported: true,
        recent_projects: vec![
            RecentProjectItemSnapshot {
                display_name: "Zircon Main UI".to_string(),
                path: "E:/Git/ZirconEngine".to_string(),
                validation: RecentProjectValidation::Valid,
                last_opened_label: "Today".to_string(),
                selected: true,
            },
            RecentProjectItemSnapshot {
                display_name: "Legacy Slate Audit".to_string(),
                path: "E:/Archives/ZirconSlateAudit".to_string(),
                validation: RecentProjectValidation::Missing,
                last_opened_label: "Missing".to_string(),
                selected: false,
            },
        ],
        form: NewProjectFormSnapshot {
            project_name: "ZirconProjectB".to_string(),
            location: "E:/Zircon/Projects".to_string(),
            project_path_preview: "E:/Zircon/Projects/ZirconProjectB".to_string(),
            template_label: "Material Slate Workspace".to_string(),
            can_create: true,
            can_open_existing: true,
            validation_message: "Name is valid after typing B.".to_string(),
        },
    };
    presented_window_from_chrome(
        EditorChromeSnapshot::build(
            data,
            &fixture.layout,
            fixture.instances.clone(),
            fixture.descriptors.clone(),
        ),
        &fixture.layout,
        &fixture.descriptors,
        width,
        height,
        &[],
        None,
    )
}

fn asset_browser_window(width: u32, height: u32) -> UiHostWindow {
    let mut fixture = default_preview_fixture();
    let page_id = MainPageId::new("page:asset-browser");
    let instance_id = ViewInstanceId::new("editor.asset_browser#1");
    fixture.instances.push(ViewInstance {
        instance_id: instance_id.clone(),
        descriptor_id: ViewDescriptorId::new("editor.asset_browser"),
        title: "Asset Browser".to_string(),
        serializable_payload: serde_json::json!({
            "source": "m3-gui-screenshot",
            "selected": "res://ui/editor/workbench_host_window.ui.toml"
        }),
        dirty: false,
        host: ViewHost::ExclusivePage(page_id.clone()),
    });
    fixture.layout = WorkbenchLayout {
        active_main_page: page_id.clone(),
        main_pages: vec![MainHostPageLayout::ExclusiveActivityWindowPage {
            id: page_id,
            title: "Asset Browser".to_string(),
            window_instance: instance_id,
        }],
        drawers: BTreeMap::new(),
        activity_windows: BTreeMap::new(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };

    let mut data = fixture.editor.clone().into_snapshot();
    let asset_workspace = m3_asset_workspace();
    data.asset_activity = asset_workspace.clone();
    data.asset_browser = asset_workspace;
    data.status_line = "Asset Browser M3 screenshot gate".to_string();
    data.project_path = "E:/Git/ZirconEngine".to_string();
    data.session_mode = EditorSessionMode::Project;
    data.project_open = true;
    presented_window_from_chrome(
        EditorChromeSnapshot::build(
            data,
            &fixture.layout,
            fixture.instances.clone(),
            fixture.descriptors.clone(),
        ),
        &fixture.layout,
        &fixture.descriptors,
        width,
        height,
        &[],
        None,
    )
}

fn presented_window_from_fixture(
    fixture: &PreviewFixture,
    width: u32,
    height: u32,
    preset_names: &[String],
    active_preset_name: Option<&str>,
) -> UiHostWindow {
    let chrome = fixture.build_chrome();
    presented_window_from_chrome(
        chrome,
        &fixture.layout,
        &fixture.descriptors,
        width,
        height,
        preset_names,
        active_preset_name,
    )
}

fn presented_window_from_chrome(
    chrome: EditorChromeSnapshot,
    layout: &WorkbenchLayout,
    descriptors: &[ViewDescriptor],
    width: u32,
    height: u32,
    preset_names: &[String],
    active_preset_name: Option<&str>,
) -> UiHostWindow {
    let model = WorkbenchViewModel::build(&chrome);
    let shell_size = ShellSizePx::new(width as f32, height as f32);
    let metrics = WorkbenchChromeMetrics::default();
    let geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        layout,
        descriptors,
        shell_size,
        &metrics,
        None,
    );
    let floating_window_projection_bundle =
        build_floating_window_projection_bundle(&model, None, &metrics, &[]);
    let ui_asset_panes: BTreeMap<String, UiAssetEditorPanePresentation> = BTreeMap::new();
    let animation_panes: BTreeMap<String, AnimationEditorPanePresentation> = BTreeMap::new();
    let module_plugins = ModulePluginsPaneViewData::default();
    let build_export = BuildExportPaneViewData::default();
    let ui = UiHostWindow::new().expect("workbench shell should instantiate for screenshot");

    ui.show()
        .expect("workbench shell should show for screenshot capture");
    ui.window().set_size(PhysicalSize::new(width, height));
    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        preset_names,
        active_preset_name,
        &ui_asset_panes,
        &animation_panes,
        None,
        &module_plugins,
        &build_export,
        None,
        &floating_window_projection_bundle,
        None,
    );
    ui
}

fn release_first_document_tab_drag(ui: &UiHostWindow) {
    let presentation = ui.get_host_presentation();
    let document = &presentation.host_scene_data.document_dock;
    let tab = document
        .tab_frames
        .row_data(0)
        .expect("default workbench screenshot should expose a document tab");
    let start_x = document.region_frame.x + tab.frame.x + tab.frame.width * 0.5;
    let start_y = document.region_frame.y + tab.frame.y + tab.frame.height * 0.5;
    let drop_x = document.region_frame.x + document.content_frame.x + 132.0;
    let drop_y = document.region_frame.y + document.content_frame.y + 74.0;

    ui.dispatch_native_primary_press_for_test(start_x, start_y);
    ui.dispatch_native_pointer_move_for_test(drop_x, drop_y);
    ui.dispatch_native_primary_release_for_test(drop_x, drop_y);

    let drag_state = ui.global::<UiHostContext>().get_drag_state();
    assert!(
        drag_state.drag_tab_id.is_empty() && !drag_state.drag_active,
        "drag capture must clear before the no-residue screenshot is accepted"
    );
}

fn save_window_snapshot(ui: &UiHostWindow, filename: &str) -> PathBuf {
    let snapshot = ui
        .window()
        .take_snapshot()
        .unwrap_or_else(|error| panic!("software renderer should capture {filename}: {error}"));
    let output_dir = visual_layout_output_dir();
    std::fs::create_dir_all(&output_dir).expect("visual-layout output directory should exist");
    let output_path = output_dir.join(filename);

    image::save_buffer_with_format(
        &output_path,
        snapshot.as_bytes(),
        snapshot.width(),
        snapshot.height(),
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .unwrap_or_else(|error| panic!("{filename} should be written as PNG: {error}"));

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
    output_path
}

fn visual_layout_output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should live under the repository root")
        .join("target")
        .join("visual-layout")
}

fn host_window_layout_for_visual_artifact(width: f32, height: f32) -> HostWindowLayoutData {
    HostWindowLayoutData {
        center_band_frame: frame(0.0, 38.0, width, height - 62.0),
        status_bar_frame: frame(0.0, height - 24.0, width, 24.0),
        left_region_frame: frame(0.0, 38.0, 198.0, height - 62.0),
        document_region_frame: frame(198.0, 38.0, width - 198.0, height - 62.0),
        viewport_content_frame: frame(214.0, 66.0, width - 230.0, height - 118.0),
        ..HostWindowLayoutData::default()
    }
}

fn nested_menu_chrome_for_visual_artifact() -> HostMenuChromeData {
    HostMenuChromeData {
        top_bar_height_px: 25.0,
        menu_frames: crate::ui::layouts::common::model_rc(vec![HostChromeControlFrameData {
            control_id: "MenuSlotTools".into(),
            frame: frame(72.0, 2.0, 64.0, 22.0),
        }]),
        menus: crate::ui::layouts::common::model_rc(vec![HostMenuChromeMenuData {
            label: "Tools".into(),
            popup_width_px: 184.0,
            popup_height_px: 92.0,
            items: crate::ui::layouts::common::model_rc(vec![
                HostMenuChromeItemData {
                    label: "Weather".into(),
                    shortcut: ">".into(),
                    enabled: true,
                    children: crate::ui::layouts::common::model_rc(vec![
                        HostMenuChromeItemData {
                            label: "Refresh Clouds".into(),
                            action_id: "Weather.CloudLayer.Refresh".into(),
                            shortcut: "Ctrl+Alt+R".into(),
                            enabled: true,
                            ..HostMenuChromeItemData::default()
                        },
                        HostMenuChromeItemData {
                            label: "Bake Probe Preview".into(),
                            action_id: "Weather.Probe.BakePreview".into(),
                            enabled: true,
                            ..HostMenuChromeItemData::default()
                        },
                    ]),
                    ..HostMenuChromeItemData::default()
                },
                HostMenuChromeItemData {
                    label: "Diagnostics".into(),
                    action_id: "Tools.Diagnostics.Open".into(),
                    shortcut: "Ctrl+Shift+D".into(),
                    enabled: true,
                    ..HostMenuChromeItemData::default()
                },
            ]),
            popup_nodes: crate::ui::layouts::common::model_rc(vec![
                template_node("NestedMenuPopupPanel", "Panel", "", 0.0, 0.0, 184.0, 92.0),
                template_node(
                    "NestedMenuPopupItem0",
                    "Panel",
                    "Weather",
                    6.0,
                    6.0,
                    172.0,
                    26.0,
                ),
                template_node(
                    "NestedMenuPopupItem1",
                    "Panel",
                    "Diagnostics",
                    6.0,
                    36.0,
                    172.0,
                    26.0,
                ),
            ]),
        }]),
        ..HostMenuChromeData::default()
    }
}

fn template_node(
    control_id: &str,
    role: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: format!("{control_id}.node").into(),
        control_id: control_id.into(),
        role: role.into(),
        text: text.into(),
        surface_variant: "panel".into(),
        border_width: 1.0,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn window_menu_preset_names() -> Vec<String> {
    (0..24).map(|index| format!("Preset {index:02}")).collect()
}

fn m3_asset_workspace() -> AssetWorkspaceSnapshot {
    AssetWorkspaceSnapshot {
        project_name: "Zircon M3 Visual".to_string(),
        project_root: "E:/Git/ZirconEngine".to_string(),
        assets_root: "zircon_editor/assets".to_string(),
        library_root: "zircon_runtime/assets".to_string(),
        default_scene_uri: "res://scenes/editor_preview.zscene".to_string(),
        catalog_revision: 42,
        folder_tree: m3_asset_folders(),
        visible_folders: m3_asset_folders(),
        visible_assets: vec![
            asset_item(
                "asset-ui-layout",
                "res://ui/editor/workbench_host_window.ui.toml",
                "workbench_host_window.ui.toml",
                "ui.toml",
                ResourceKind::UiLayout,
                true,
            ),
            asset_item(
                "asset-theme-base",
                "res://ui/theme/editor_base.ui.toml",
                "editor_base.ui.toml",
                "ui.toml",
                ResourceKind::UiStyle,
                false,
            ),
            asset_item(
                "asset-folder-open-svg",
                "res://icons/ionicons/folder-open-outline.svg",
                "folder-open-outline.svg",
                "svg",
                ResourceKind::Texture,
                false,
            ),
        ],
        selected_folder_id: Some("folder-ui".to_string()),
        selected_asset_uuid: Some("asset-ui-layout".to_string()),
        selection: AssetSelectionSnapshot {
            uuid: Some("asset-ui-layout".to_string()),
            display_name: "workbench_host_window.ui.toml".to_string(),
            locator: "res://ui/editor/workbench_host_window.ui.toml".to_string(),
            kind: Some(ResourceKind::UiLayout),
            preview_artifact_path: "target/visual-layout/editor-window-m3-workbench-900x620.png"
                .to_string(),
            meta_path: "zircon_editor/assets/ui/editor/workbench_host_window.ui.toml".to_string(),
            adapter_key: "runtime-ui-template".to_string(),
            diagnostics: vec!["SVG icons resolve through scalable template metadata.".to_string()],
            resource_state: Some(ResourceState::Ready),
            resource_revision: Some(42),
            references: Vec::new(),
            used_by: Vec::new(),
        },
        ..AssetWorkspaceSnapshot::default()
    }
}

fn m3_asset_folders() -> Vec<AssetFolderSnapshot> {
    vec![
        AssetFolderSnapshot {
            folder_id: "folder-assets".to_string(),
            parent_folder_id: None,
            display_name: "Assets".to_string(),
            recursive_asset_count: 3,
            depth: 0,
            selected: false,
        },
        AssetFolderSnapshot {
            folder_id: "folder-ui".to_string(),
            parent_folder_id: Some("folder-assets".to_string()),
            display_name: "ui".to_string(),
            recursive_asset_count: 2,
            depth: 1,
            selected: true,
        },
        AssetFolderSnapshot {
            folder_id: "folder-icons".to_string(),
            parent_folder_id: Some("folder-assets".to_string()),
            display_name: "icons".to_string(),
            recursive_asset_count: 1,
            depth: 1,
            selected: false,
        },
    ]
}

fn asset_item(
    uuid: &str,
    locator: &str,
    file_name: &str,
    extension: &str,
    kind: ResourceKind,
    selected: bool,
) -> AssetItemSnapshot {
    AssetItemSnapshot {
        uuid: uuid.to_string(),
        locator: locator.to_string(),
        display_name: file_name.to_string(),
        file_name: file_name.to_string(),
        extension: extension.to_string(),
        kind,
        preview_artifact_path: String::new(),
        dirty: false,
        diagnostics: Vec::new(),
        selected,
        resource_state: Some(ResourceState::Ready),
        resource_revision: Some(42),
    }
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}

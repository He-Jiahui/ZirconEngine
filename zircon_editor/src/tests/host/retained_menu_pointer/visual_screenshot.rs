use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::ui::retained_host::primitives::{Color, PhysicalSize, SharedString};
use zircon_runtime_interface::resource::{ResourceKind, ResourceState};

use crate::ui::animation_editor::AnimationEditorPanePresentation;
use crate::ui::asset_editor::UiAssetEditorPanePresentation;
use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, ModulePluginsPaneViewData,
};
use crate::ui::retained_host::callback_dispatch::{
    load_startup_builtin_template_runtime, BuiltinWorkbenchWindowLayoutFrames,
    BuiltinWorkbenchWindowTemplateSurfaceBridge,
};
use crate::ui::retained_host::floating_window_projection::build_floating_window_projection_bundle;
use crate::ui::retained_host::{
    apply_presentation, paint_host_frame_for_test, paint_template_nodes_for_test_with_background,
    FrameRect, HostChromeControlFrameData, HostChromeTabData, HostClosePromptData,
    HostMenuChromeData, HostMenuChromeItemData, HostMenuChromeMenuData, HostMenuStateData,
    HostPageOverflowMenuStateData, HostWindowLayoutData, TabData, TemplateNodeFrameData,
    TemplatePaneNodeData, UiHostContext, UiHostWindow,
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
    AssetFolderSnapshot, AssetItemSnapshot, AssetReferenceSnapshot, AssetSelectionSnapshot,
    AssetSubassetSnapshot, AssetUtilityTab, AssetViewMode, AssetWorkspaceSnapshot,
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
use zircon_runtime_interface::ui::layout::UiSize;

const SCROLLED_WINDOW_POPUP_SCREENSHOT: &str =
    "editor-window-20260429-window-popup-scrolled-900x620.png";
const CLOSE_PROMPT_SCREENSHOT: &str = "editor-window-20260507-close-prompt-900x620.png";
const NESTED_MENU_POPUP_SCREENSHOT: &str = "editor-window-20260507-nested-menu-popup-900x620.png";
const M3_WELCOME_INPUT_SCREENSHOT: &str = "editor-window-m3-welcome-input-900x620.png";
const M3_WORKBENCH_SCREENSHOT: &str = "editor-window-m3-workbench-900x620.png";
const M3_ASSET_BROWSER_SCREENSHOT: &str = "editor-window-m3-asset-browser-900x620.png";
const M3_ASSET_BROWSER_LIST_SCREENSHOT: &str = "editor-window-m3-asset-browser-list-900x620.png";
const M3_DRAWER_SCREENSHOT: &str = "editor-window-m3-assets-drawer-900x620.png";
const M3_MENU_POPUP_SCREENSHOT: &str = "editor-window-m3-menu-popup-svg-icons-900x620.png";
const M3_HOST_PAGE_OVERFLOW_SCREENSHOT: &str = "editor-window-m3-host-page-overflow-420x260.png";
const M3_DRAG_AFTER_RELEASE_SCREENSHOT: &str = "editor-window-m3-drag-after-release-900x620.png";
const M3_SVG_ICON_SMALL_SCREENSHOT: &str = "editor-window-m3-svg-icon-scale-small-640x420.png";
const M3_SVG_ICON_LARGE_SCREENSHOT: &str = "editor-window-m3-svg-icon-scale-large-1260x780.png";
const WORKBENCH_COMPONENT_ATLAS_SCREENSHOT: &str =
    "editor-components-workbench-slate-atlas-900x620.png";
const REFERENCE_WORKBENCH_MIN_DOCUMENT_WIDTH_FRACTION: f32 = 0.55;

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
        1.0,
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
    let workbench_window_bridge = workbench_window_bridge_for_visual_artifact(&model, 900, 620);
    let componentized_workbench_layout_frames = workbench_window_bridge.layout_frames();
    assert_visible_workbench_layout_frames(&componentized_workbench_layout_frames, 900, 620);

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
        Some(workbench_window_bridge.host_projection()),
        componentized_workbench_layout_frames,
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
    let output_path = visual_layout_output_path(SCROLLED_WINDOW_POPUP_SCREENSHOT);

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
        1.0,
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
    let workbench_window_bridge = workbench_window_bridge_for_visual_artifact(&model, 900, 620);
    let componentized_workbench_layout_frames = workbench_window_bridge.layout_frames();
    assert_visible_workbench_layout_frames(&componentized_workbench_layout_frames, 900, 620);

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
        Some(workbench_window_bridge.host_projection()),
        componentized_workbench_layout_frames,
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
    let output_path = visual_layout_output_path(CLOSE_PROMPT_SCREENSHOT);

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

    let workbench = reference_asset_workbench_window(900, 620);
    assert_reference_asset_workbench_layout(&workbench, 900);
    save_window_snapshot(&workbench, M3_WORKBENCH_SCREENSHOT);

    let welcome = welcome_input_window(900, 620);
    save_window_snapshot(&welcome, M3_WELCOME_INPUT_SCREENSHOT);

    let asset_browser = asset_browser_window(900, 620);
    assert_asset_browser_compact_visual_layout(&asset_browser);
    save_window_snapshot(&asset_browser, M3_ASSET_BROWSER_SCREENSHOT);

    let asset_browser_list = asset_browser_list_window(900, 620);
    assert_asset_browser_list_visual_layout(&asset_browser_list);
    save_window_snapshot(&asset_browser_list, M3_ASSET_BROWSER_LIST_SCREENSHOT);

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

#[test]
#[ignore = "writes host-page overflow screenshot artifact for component style review"]
fn capture_host_page_overflow_menu_visual_artifact() {
    std::env::set_var("SLINT_BACKEND", "software");

    let ui = asset_browser_window(420, 260);
    set_host_page_overflow_visual_state(
        &ui,
        HostPageOverflowMenuStateData {
            open: false,
            hovered_page_index: -1,
        },
    );
    let closed_presentation = ui.get_host_presentation();
    let closed_bytes = paint_host_frame_for_test(420, 260, &closed_presentation);

    set_host_page_overflow_visual_state(
        &ui,
        HostPageOverflowMenuStateData {
            open: true,
            hovered_page_index: 2,
        },
    );
    let opened_presentation = ui.get_host_presentation();
    let opened_bytes = paint_host_frame_for_test(420, 260, &opened_presentation);

    let popup_probe_frame = frame(48.0, 58.0, 178.0, 92.0);
    let changed_pixels = changed_snapshot_pixel_count_in_frame(
        &closed_bytes,
        &opened_bytes,
        420,
        260,
        popup_probe_frame,
    );
    assert!(
        changed_pixels > 900,
        "opened host-page overflow menu should repaint a visible popup area: changed_pixels={changed_pixels}"
    );

    let output_path = visual_layout_output_path(M3_HOST_PAGE_OVERFLOW_SCREENSHOT);
    image::save_buffer_with_format(
        &output_path,
        &opened_bytes,
        420,
        260,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("host-page overflow screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

#[test]
#[ignore = "writes local workbench component visual atlas for bottom-up style review"]
fn capture_workbench_component_slate_atlas_visual_artifact() {
    let width = 900;
    let height = 620;
    let bytes = paint_template_nodes_for_test_with_background(
        width,
        height,
        [17, 20, 22, 255],
        crate::ui::layouts::common::model_rc(workbench_component_atlas_nodes()),
    );
    let output_path = visual_layout_output_path(WORKBENCH_COMPONENT_ATLAS_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        width,
        height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("workbench component atlas screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn workbench_component_atlas_nodes() -> Vec<TemplatePaneNodeData> {
    let mut nodes = vec![
        atlas_surface("AtlasRoot", "shell", 0.0, 0.0, 900.0, 620.0),
        atlas_label(
            "AtlasTitle",
            "Workbench Component Style Atlas",
            22.0,
            20.0,
            360.0,
            22.0,
            13.0,
            "",
        ),
        atlas_label(
            "AtlasSubtitle",
            "Buttons, text, inputs, image containers, rows, tables, bars and popups",
            22.0,
            42.0,
            560.0,
            18.0,
            10.0,
            "muted",
        ),
        atlas_surface("AtlasButtonsPanel", "panel", 18.0, 78.0, 272.0, 190.0),
        atlas_surface("AtlasInputsPanel", "panel", 306.0, 78.0, 276.0, 190.0),
        atlas_surface("AtlasRowsPanel", "panel", 598.0, 78.0, 284.0, 190.0),
        atlas_surface("AtlasComplexPanel", "panel", 18.0, 286.0, 420.0, 266.0),
        atlas_surface("AtlasContainersPanel", "panel", 454.0, 286.0, 428.0, 266.0),
        atlas_surface("AtlasStatusBar", "inset", 0.0, 578.0, 900.0, 42.0),
    ];

    nodes.extend([
        atlas_label(
            "AtlasButtonsTitle",
            "Buttons",
            34.0,
            96.0,
            220.0,
            18.0,
            11.0,
            "",
        ),
        atlas_button(
            "WorkbenchPrimaryButton",
            "Primary",
            "primary",
            34.0,
            128.0,
            112.0,
            26.0,
        ),
        atlas_button(
            "WorkbenchSecondaryButton",
            "Secondary",
            "secondary",
            156.0,
            128.0,
            112.0,
            26.0,
        ),
        atlas_button(
            "WorkbenchTertiaryButton",
            "Tertiary",
            "tertiary",
            34.0,
            162.0,
            112.0,
            26.0,
        ),
        atlas_button(
            "WorkbenchDangerButton",
            "Danger",
            "danger",
            156.0,
            162.0,
            112.0,
            26.0,
        ),
        atlas_button_state(
            "WorkbenchHoverButton",
            "Hover",
            "secondary",
            34.0,
            204.0,
            70.0,
            24.0,
            "hover",
        ),
        atlas_button_state(
            "WorkbenchPressedButton",
            "Pressed",
            "secondary",
            112.0,
            204.0,
            76.0,
            24.0,
            "pressed",
        ),
        atlas_button_state(
            "WorkbenchDisabledButton",
            "Disabled",
            "secondary",
            196.0,
            204.0,
            72.0,
            24.0,
            "disabled",
        ),
    ]);

    nodes.extend([
        atlas_label(
            "AtlasInputsTitle",
            "Inputs And Selection",
            322.0,
            96.0,
            220.0,
            18.0,
            11.0,
            "",
        ),
        atlas_field(
            "WorkbenchInputSearch",
            "Search assets...",
            322.0,
            128.0,
            238.0,
            28.0,
            "",
        ),
        atlas_field(
            "WorkbenchInputFocused",
            "Focused field",
            322.0,
            164.0,
            238.0,
            28.0,
            "focus",
        ),
        atlas_dropdown(
            "WorkbenchDropdownAtlas",
            "Kind: Mesh",
            322.0,
            200.0,
            116.0,
            28.0,
            "",
        ),
        atlas_selection(
            "WorkbenchCheckboxAtlas",
            "Checkbox",
            454.0,
            199.0,
            104.0,
            28.0,
            "checkbox",
            true,
        ),
        atlas_selection(
            "WorkbenchRadioAtlas",
            "Radio",
            322.0,
            234.0,
            96.0,
            26.0,
            "radio",
            true,
        ),
        atlas_selection(
            "WorkbenchToggleAtlas",
            "Snap",
            448.0,
            234.0,
            112.0,
            26.0,
            "toggle",
            true,
        ),
    ]);

    nodes.extend([
        atlas_label(
            "AtlasRowsTitle",
            "Rows And Lists",
            614.0,
            96.0,
            220.0,
            18.0,
            11.0,
            "",
        ),
        atlas_list_row(
            "WorkbenchListAsset0",
            "Neutral list row",
            614.0,
            126.0,
            246.0,
            30.0,
            "",
        ),
        atlas_list_row(
            "WorkbenchListAsset1",
            "Selected list row",
            614.0,
            160.0,
            246.0,
            30.0,
            "selected",
        ),
        atlas_tree_row(
            "WorkbenchSceneAssetItem",
            "Scene tree row",
            614.0,
            202.0,
            246.0,
            28.0,
            1,
            true,
        ),
        atlas_tree_row(
            "WorkbenchSceneLightItem",
            "Child row hover",
            614.0,
            234.0,
            246.0,
            28.0,
            2,
            false,
        ),
    ]);

    nodes.extend([
        atlas_label(
            "AtlasComplexTitle",
            "Complex Content",
            34.0,
            304.0,
            240.0,
            18.0,
            11.0,
            "",
        ),
        atlas_segmented(
            "WorkbenchSegmentedAtlas",
            &["All", "Selected", "Recent"],
            "Selected",
            34.0,
            338.0,
            252.0,
            34.0,
        ),
        atlas_table_row(
            "WorkbenchTableHeader",
            &["Name", "Type", "Size", "Modified"],
            34.0,
            390.0,
            370.0,
            28.0,
            false,
        ),
        atlas_table_row(
            "WorkbenchTableSelected",
            &["Box_01.mesh", "Mesh", "2.4 MB", "2m ago"],
            34.0,
            418.0,
            370.0,
            30.0,
            true,
        ),
        atlas_table_row(
            "WorkbenchTableRowAsset",
            &["M_Metal.zmat", "Material", "512 KB", "10m ago"],
            34.0,
            448.0,
            370.0,
            30.0,
            false,
        ),
        atlas_label(
            "AtlasProgressLabel",
            "Progress",
            34.0,
            492.0,
            70.0,
            16.0,
            10.0,
            "muted",
        ),
        atlas_progress("WorkbenchProgressAtlas", 0.64, 112.0, 496.0, 292.0, 12.0),
    ]);

    nodes.extend([
        atlas_label(
            "AtlasContainersTitle",
            "Containers, Images And Overlays",
            470.0,
            304.0,
            280.0,
            18.0,
            11.0,
            "",
        ),
        atlas_surface("AtlasImageCard", "inset", 470.0, 338.0, 132.0, 96.0),
        atlas_surface(
            "AtlasImagePreview",
            "asset-preview-visual",
            486.0,
            354.0,
            100.0,
            48.0,
        ),
        atlas_label(
            "AtlasImageLabel",
            "Image preview",
            492.0,
            414.0,
            94.0,
            18.0,
            10.0,
            "muted",
        ),
        atlas_surface("AtlasPopup", "popup", 624.0, 338.0, 220.0, 96.0),
        atlas_label(
            "AtlasPopupTitle",
            "Popup / Picker",
            640.0,
            354.0,
            160.0,
            18.0,
            11.0,
            "",
        ),
        atlas_field(
            "WorkbenchInputPopupFilter",
            "Filter rows...",
            640.0,
            380.0,
            184.0,
            28.0,
            "",
        ),
        atlas_list_row(
            "WorkbenchListPopupSelected",
            "Interactive option",
            640.0,
            414.0,
            184.0,
            28.0,
            "selected",
        ),
        atlas_tooltip(
            "WorkbenchTooltipAtlas",
            "Tooltip",
            "Host route",
            470.0,
            448.0,
            132.0,
            70.0,
        ),
        atlas_dialog(
            "WorkbenchDialogAtlas",
            "Unsaved asset",
            "Apply material changes?",
            624.0,
            446.0,
            220.0,
            104.0,
            "warning",
        ),
    ]);

    nodes.extend([
        atlas_status_signal("WorkbenchStatusReady", "Ready", 2.0, 578.0, 104.0, 42.0),
        atlas_status_signal(
            "WorkbenchStatusWarnings",
            "2 Warnings",
            110.0,
            578.0,
            128.0,
            42.0,
        ),
        atlas_status_signal(
            "WorkbenchStatusMessages",
            "0 Messages",
            238.0,
            578.0,
            132.0,
            42.0,
        ),
        atlas_status_chip(
            "WorkbenchStatusGrid",
            "Grid: 10 cm",
            634.0,
            585.0,
            92.0,
            28.0,
        ),
        atlas_status_chip("WorkbenchStatusSnap", "Snap: On", 734.0, 585.0, 82.0, 28.0),
        atlas_status_chip("WorkbenchStatusZoom", "100%", 824.0, 585.0, 52.0, 28.0),
    ]);

    nodes
}

fn atlas_status_signal(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_progress(
    control_id: &str,
    value_percent: f32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Progress".into(),
        component_role: "progress-bar".into(),
        value_percent,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_tooltip(
    control_id: &str,
    text: &str,
    label_text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Tooltip".into(),
        component_role: "tooltip".into(),
        surface_variant: "workbench-tooltip".into(),
        text: text.into(),
        label_text: label_text.into(),
        value_number: 8.0,
        value_color: Color::from_rgb_u8(23, 28, 32),
        label_color: Color::from_rgb_u8(168, 179, 184),
        icon_color: Color::from_rgb_u8(37, 156, 167),
        layout_icon_size: 16.0,
        layout_content_offset_y: 48.0,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_dialog(
    control_id: &str,
    title: &str,
    message: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    severity: &str,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "ConfirmDialog".into(),
        component_role: "confirm-dialog".into(),
        component_variant: severity.into(),
        surface_variant: "workbench-dialog".into(),
        text: title.into(),
        value_text: message.into(),
        popup_open: true,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_status_chip(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_surface(
    control_id: &str,
    surface_variant: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Panel".into(),
        surface_variant: surface_variant.into(),
        border_width: 1.0,
        corner_radius: 4.0,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_label(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    font_size: f32,
    tone: &str,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        font_size,
        text_tone: tone.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_button(
    control_id: &str,
    text: &str,
    variant: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Button".into(),
        component_role: "button".into(),
        text: text.into(),
        button_variant: variant.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_button_state(
    control_id: &str,
    text: &str,
    variant: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: &str,
) -> TemplatePaneNodeData {
    let mut node = atlas_button(control_id, text, variant, x, y, width, height);
    match state {
        "hover" => node.hovered = true,
        "pressed" => node.pressed = true,
        "disabled" => node.disabled = true,
        _ => {}
    }
    node
}

fn atlas_field(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: &str,
) -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Input".into(),
        component_role: "text-input".into(),
        text: text.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    };
    if state == "focus" {
        node.focused = true;
    }
    node
}

fn atlas_dropdown(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: &str,
) -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Dropdown".into(),
        component_role: "dropdown".into(),
        text: text.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    };
    if state == "open" {
        node.popup_open = true;
    }
    node
}

fn atlas_selection(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    family: &str,
    checked: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "SelectionControl".into(),
        component_role: family.into(),
        text: text.into(),
        checked,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_segmented(
    control_id: &str,
    options: &[&str],
    selected: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "SegmentedControl".into(),
        component_role: "segmented-control".into(),
        value_text: selected.into(),
        options: crate::ui::layouts::common::model_rc(
            options
                .iter()
                .map(|option| SharedString::from(*option))
                .collect::<Vec<_>>(),
        ),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_list_row(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: &str,
) -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "ListRow".into(),
        component_role: "list-row".into(),
        text: text.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    };
    match state {
        "selected" => node.selected = true,
        "hover" => node.hovered = true,
        _ => {}
    }
    node
}

fn atlas_tree_row(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    depth: i32,
    selected: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "TreeRow".into(),
        component_role: "tree-row".into(),
        text: text.into(),
        tree_depth: depth,
        expanded: depth == 1,
        hovered: !selected,
        selected,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_table_row(
    control_id: &str,
    cells: &[&str],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    selected: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "TableRow".into(),
        component_role: "table-row".into(),
        selected,
        options: crate::ui::layouts::common::model_rc(
            cells
                .iter()
                .map(|cell| SharedString::from(*cell))
                .collect::<Vec<_>>(),
        ),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
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
    asset_browser_window_with_workspace(width, height, m3_asset_workspace())
}

fn asset_browser_list_window(width: u32, height: u32) -> UiHostWindow {
    let mut workspace = m3_asset_workspace();
    workspace.view_mode = AssetViewMode::List;
    asset_browser_window_with_workspace(width, height, workspace)
}

fn asset_browser_window_with_workspace(
    width: u32,
    height: u32,
    asset_workspace: AssetWorkspaceSnapshot,
) -> UiHostWindow {
    let mut fixture = default_preview_fixture();
    let page_id = MainPageId::new("page:asset-browser");
    let instance_id = ViewInstanceId::new("editor.asset_browser#1");
    fixture.instances.push(ViewInstance {
        instance_id: instance_id.clone(),
        descriptor_id: ViewDescriptorId::new("editor.asset_browser"),
        title: "Asset Browser".to_string(),
        serializable_payload: serde_json::json!({
            "source": "m3-gui-screenshot",
            "selected": "res://ui/editor/workbench_page_chrome.zui"
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

fn reference_asset_workbench_window(width: u32, height: u32) -> UiHostWindow {
    asset_browser_window(width, height)
}

fn assert_reference_asset_workbench_layout(ui: &UiHostWindow, width: u32) {
    let presentation = ui.get_host_presentation();
    let document = presentation.host_layout.document_region_frame;
    let viewport = presentation.host_layout.viewport_content_frame;
    let min_document_width = width as f32 * REFERENCE_WORKBENCH_MIN_DOCUMENT_WIDTH_FRACTION;

    assert!(
        document.width >= min_document_width,
        "reference workbench screenshot should keep the main document readable: {document:?}"
    );
    assert!(
        viewport.x >= document.x
            && viewport.y >= document.y
            && viewport.x + viewport.width <= document.x + document.width + 1.0
            && viewport.y + viewport.height <= document.y + document.height + 1.0,
        "reference workbench viewport should stay inside the document region: document={document:?}, viewport={viewport:?}"
    );
}

fn assert_asset_browser_compact_visual_layout(ui: &UiHostWindow) {
    let presentation = ui.get_host_presentation();
    let pane = &presentation.host_scene_data.document_dock.pane;
    assert_eq!(pane.kind.as_str(), "AssetBrowser");

    let nodes = &pane.asset_browser.nodes;
    let content = find_template_node(nodes, "AssetBrowserContentPanel");
    let table = find_template_node(nodes, "AssetBrowserAssetTablePanel");
    let grid = find_template_node(nodes, "AssetBrowserThumbGridPanel");
    let first_thumb = find_template_node(nodes, "AssetBrowserThumbCard01");
    let second_thumb = find_template_node(nodes, "AssetBrowserThumbCard02");
    let seventh_thumb = find_template_node(nodes, "AssetBrowserThumbCard07");

    assert_eq!(
        visible_template_node_count(nodes, "AssetBrowserContentPanel"),
        1,
        "asset browser compact content panel should not leave a second visible projected container"
    );
    assert_eq!(
        visible_template_node_count(nodes, "AssetBrowserAssetTablePanel"),
        0,
        "thumbnail asset browser compact view should hide the list table panel"
    );
    assert!(
        table.frame.height == 0.0,
        "thumbnail asset browser table frame should collapse"
    );
    assert!(
        grid.frame.width > content.frame.width * 0.75 && grid.frame.height >= 86.0,
        "thumbnail asset browser compact view should expose an adaptive content grid"
    );
    assert!(
        first_thumb.frame.width >= 104.0 && second_thumb.frame.x > first_thumb.frame.x,
        "thumbnail asset cards should lay out horizontally from available content width"
    );
    assert!(
        seventh_thumb.frame.y > first_thumb.frame.y,
        "thumbnail asset browser should use recovered summary space for a second asset row"
    );
    assert_eq!(
        visible_template_node_count(nodes, "AssetBrowserContentPreviewCard"),
        0,
        "thumbnail asset browser compact view should keep selection feedback inside the tile grid"
    );
}

fn assert_asset_browser_list_visual_layout(ui: &UiHostWindow) {
    let presentation = ui.get_host_presentation();
    let pane = &presentation.host_scene_data.document_dock.pane;
    assert_eq!(pane.kind.as_str(), "AssetBrowser");

    let nodes = &pane.asset_browser.nodes;
    let content = find_template_node(nodes, "AssetBrowserContentPanel");
    let table = find_template_node(nodes, "AssetBrowserAssetTablePanel");
    let header = find_template_node(nodes, "WorkbenchAssetBrowserTableHeader");
    let selected_row = find_template_node(nodes, "WorkbenchAssetBrowserAssetRow01");
    let next_row = find_template_node(nodes, "WorkbenchAssetBrowserAssetRow02");
    let preview = find_template_node(nodes, "AssetBrowserContentPreviewCard");
    let preview_visual = find_template_node(nodes, "AssetBrowserContentPreviewVisual");
    let preview_name = find_template_node(nodes, "AssetBrowserContentPreviewName");
    let preview_name_continuation =
        find_template_node(nodes, "AssetBrowserContentPreviewNameContinuation");

    assert_eq!(
        visible_template_node_count(nodes, "AssetBrowserAssetTablePanel"),
        1,
        "list asset browser should expose one visible retained table panel"
    );
    assert!(
        table.frame.width > content.frame.width * 0.75 && table.frame.height >= 140.0,
        "list asset browser should keep the table readable in the content panel"
    );
    assert!(
        header.frame.height > 0.0 && selected_row.frame.y > header.frame.y,
        "list table header and rows should stack in reading order"
    );
    assert!(
        selected_row.selected && !selected_row.focused,
        "selected list row should not impersonate keyboard focus"
    );
    assert!(
        selected_row
            .text
            .as_str()
            .contains("workbench_page_chrome.zui"),
        "list table rows should preserve readable asset filenames for scan-heavy workbench lists"
    );
    assert!(
        !next_row.selected && !next_row.focused,
        "unselected list rows should remain visually idle"
    );
    assert!(
        preview.frame.height > 0.0 && preview.frame.y >= table.frame.y + table.frame.height,
        "list asset browser should retain a compact selection preview below the table"
    );
    assert_eq!(
        preview_visual.frame.width, preview_visual.frame.height,
        "selection preview visual should use a square asset icon slot rather than a wide empty pill"
    );
    assert_eq!(
        preview_visual.component_variant.as_str(),
        "asset-ui-layout",
        "selection preview should keep the selected asset type icon identity"
    );
    assert!(
        preview_name.frame.x - (preview_visual.frame.x + preview_visual.frame.width) <= 12.0,
        "selection preview title should sit close to the square asset icon slot"
    );
    assert_eq!(
        preview_name.text.as_str(),
        "workbench_page_chrome.zui",
        "selection preview should keep file-like asset names on one readable line"
    );
    assert_eq!(
        preview_name_continuation.text.as_str(),
        "",
        "file-like selection preview names should not be split like thumbnail title text"
    );
    assert_eq!(
        preview_name_continuation.frame.height, 0.0,
        "empty selection preview continuation should collapse out of the summary rhythm"
    );
    assert_eq!(
        visible_template_node_count(nodes, "AssetBrowserThumbGridPanel"),
        0,
        "list asset browser should hide the thumbnail grid panel"
    );
}

fn find_template_node(
    nodes: &crate::ui::retained_host::primitives::ModelRc<TemplatePaneNodeData>,
    control_id: &str,
) -> TemplatePaneNodeData {
    for index in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(index) else {
            continue;
        };
        if node.control_id.as_str() == control_id {
            return node;
        }
    }
    panic!("missing template node `{control_id}`");
}

fn visible_template_node_count(
    nodes: &crate::ui::retained_host::primitives::ModelRc<TemplatePaneNodeData>,
    control_id: &str,
) -> usize {
    let mut count = 0;
    for index in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(index) else {
            continue;
        };
        if node.control_id.as_str() == control_id
            && node.frame.width > 1.0
            && node.frame.height > 1.0
        {
            count += 1;
        }
    }
    count
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
        1.0,
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
    let workbench_window_bridge =
        workbench_window_bridge_for_visual_artifact(&model, width, height);
    let componentized_workbench_layout_frames = workbench_window_bridge.layout_frames();
    assert_visible_workbench_layout_frames(&componentized_workbench_layout_frames, width, height);

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
        Some(workbench_window_bridge.host_projection()),
        componentized_workbench_layout_frames,
        &floating_window_projection_bundle,
        None,
    );
    ui
}

fn workbench_window_bridge_for_visual_artifact(
    model: &WorkbenchViewModel,
    width: u32,
    height: u32,
) -> BuiltinWorkbenchWindowTemplateSurfaceBridge {
    let shell_size = UiSize::new(width as f32, height as f32);
    let runtime = Arc::new(
        load_startup_builtin_template_runtime()
            .expect("startup template runtime should load for screenshot"),
    );
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new_with_runtime(runtime, shell_size)
            .expect("workbench window template bridge should instantiate for screenshot");
    bridge
        .recompute_layout_with_workbench_model(
            shell_size,
            model,
            &WorkbenchChromeMetrics::default(),
        )
        .expect("workbench window template bridge should recompute screenshot layout");
    bridge
}

fn set_host_page_overflow_visual_state(ui: &UiHostWindow, state: HostPageOverflowMenuStateData) {
    let tabs = vec![
        host_page_tab("page:workbench", "Workbench", true),
        host_page_tab("page:assets", "Assets", false),
        host_page_tab("page:materials", "Materials", false),
        host_page_tab("page:animation", "Animation", false),
    ];
    let mut presentation = ui.get_host_presentation();

    presentation.host_scene_data.page_chrome.tabs =
        crate::ui::layouts::common::model_rc(tabs.clone());
    presentation.host_scene_data.page_chrome.tab_frames =
        crate::ui::layouts::common::model_rc(vec![HostChromeTabData {
            control_id: "HostPageWorkbench".into(),
            tab: tabs[0].clone(),
            frame: frame(68.0, 29.0, 116.0, 28.0),
            close_frame: frame(0.0, 0.0, 0.0, 0.0),
        }]);
    presentation.host_scene_data.page_chrome.overflow_frame = frame(188.0, 29.0, 34.0, 28.0);
    presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices = vec![1, 2, 3];
    presentation.host_page_overflow_menu_state = state.clone();

    ui.set_host_presentation(presentation);
    ui.global::<UiHostContext>()
        .set_host_page_overflow_menu_state(state);
}

fn host_page_tab(id: &str, title: &str, active: bool) -> TabData {
    TabData {
        id: id.into(),
        slot: SharedString::default(),
        title: title.into(),
        icon_key: SharedString::default(),
        active,
        closeable: false,
    }
}

fn changed_snapshot_pixel_count_in_frame(
    before: &[u8],
    after: &[u8],
    width: u32,
    height: u32,
    frame: FrameRect,
) -> usize {
    let start_x = frame.x.floor().max(0.0) as u32;
    let start_y = frame.y.floor().max(0.0) as u32;
    let end_x = (frame.x + frame.width).ceil().min(width as f32) as u32;
    let end_y = (frame.y + frame.height).ceil().min(height as f32) as u32;

    (start_y..end_y)
        .flat_map(|y| (start_x..end_x).map(move |x| (x, y)))
        .filter(|(x, y)| {
            let offset = ((*y as usize * width as usize) + *x as usize) * 4;
            before[offset..offset + 4] != after[offset..offset + 4]
        })
        .count()
}

fn assert_visible_workbench_layout_frames(
    frames: &BuiltinWorkbenchWindowLayoutFrames,
    width: u32,
    height: u32,
) {
    let center = frames
        .center_band_frame
        .expect("screenshot layout must expose a visible center band frame");
    let document = frames
        .document_region_frame
        .expect("screenshot layout must expose a visible document region frame");
    let viewport = frames
        .viewport_content_frame
        .expect("screenshot layout must expose a visible viewport content frame");
    let status = frames
        .status_bar_frame
        .expect("screenshot layout must expose a visible status bar frame");

    assert!(
        center.y >= 44.0 && center.width > width as f32 * 0.5,
        "screenshot center band should start below the compact toolbar: {center:?}"
    );
    assert!(
        document.y >= center.y && document.height > height as f32 * 0.45,
        "screenshot document region should live inside the center band: {document:?}"
    );
    let document_right = document.x + document.width;
    let document_bottom = document.y + document.height;
    let viewport_right = viewport.x + viewport.width;
    let viewport_bottom = viewport.y + viewport.height;
    let min_viewport_width = if width >= 800 { 96.0 } else { 8.0 };
    let min_viewport_height = if height >= 500 { 96.0 } else { 48.0 };
    assert!(
        viewport.x >= document.x
            && viewport.y >= document.y
            && viewport_right <= document_right + 1.0
            && viewport_bottom <= document_bottom + 1.0
            && viewport.width >= min_viewport_width
            && viewport.height >= min_viewport_height,
        "screenshot viewport should live inside the document region: document={document:?}, viewport={viewport:?}"
    );
    let status_bottom = status.y + status.height;
    assert!(
        (status_bottom - height as f32).abs() <= 1.0 && status.height > 20.0,
        "screenshot status bar should be anchored at the bottom: {status:?}"
    );
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
    let output_path = visual_layout_output_path(filename);

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
        .join("docs")
        .join("tests")
        .join("editor")
}

fn visual_layout_output_path(filename: &str) -> PathBuf {
    let output_dir = visual_layout_output_dir();
    std::fs::create_dir_all(&output_dir).expect("visual-layout output directory should exist");
    output_dir.join(filename)
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
                            action_id: "weather.cloud_layer.refresh".into(),
                            shortcut: "Ctrl+Alt+R".into(),
                            enabled: true,
                            ..HostMenuChromeItemData::default()
                        },
                        HostMenuChromeItemData {
                            label: "Bake Probe Preview".into(),
                            action_id: "weather.probe.bake_preview".into(),
                            enabled: true,
                            ..HostMenuChromeItemData::default()
                        },
                    ]),
                    ..HostMenuChromeItemData::default()
                },
                HostMenuChromeItemData {
                    label: "Diagnostics".into(),
                    action_id: "tools.diagnostics.open".into(),
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
        view_mode: AssetViewMode::Thumbnail,
        utility_tab: AssetUtilityTab::Preview,
        search_query: "workbench".to_string(),
        folder_tree: m3_asset_folders(),
        visible_folders: m3_asset_folders(),
        visible_assets: vec![
            asset_item(
                "asset-ui-layout",
                "res://ui/editor/workbench_page_chrome.zui",
                "workbench_page_chrome.zui",
                "zui",
                ResourceKind::UiLayout,
                true,
            ),
            asset_item(
                "asset-theme-base",
                "res://ui/theme/editor_base.zui",
                "editor_base.zui",
                "zui",
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
            asset_item(
                "asset-accessibility-audit",
                "res://ui/editor/components/workbench/modules/extensions/ui/workbench_extension_accessibility_workspace.zui",
                "workbench_extension_accessibility_workspace.zui",
                "zui",
                ResourceKind::UiWidget,
                false,
            ),
            asset_item(
                "asset-material-workspace",
                "res://ui/editor/components/workbench/modules/core/rendering/workbench_material_workspace.zui",
                "workbench_material_workspace.zui",
                "zui",
                ResourceKind::MaterialGraph,
                false,
            ),
            asset_item(
                "asset-scene-preview",
                "res://scenes/editor_preview.zscene",
                "editor_preview.zscene",
                "zscene",
                ResourceKind::Scene,
                false,
            ),
            asset_item(
                "asset-shader-unlit",
                "res://shaders/ui/unlit.zshader",
                "unlit.zshader",
                "zshader",
                ResourceKind::Shader,
                false,
            ),
            asset_item(
                "asset-player-prefab",
                "res://prefabs/player_start.prefab",
                "player_start.prefab",
                "prefab",
                ResourceKind::Prefab,
                false,
            ),
        ],
        selected_folder_id: Some("folder-ui".to_string()),
        selected_asset_uuid: Some("asset-ui-layout".to_string()),
        selection: AssetSelectionSnapshot {
            uuid: Some("asset-ui-layout".to_string()),
            display_name: "workbench_page_chrome.zui".to_string(),
            locator: "res://ui/editor/workbench_page_chrome.zui".to_string(),
            kind: Some(ResourceKind::UiLayout),
            preview_artifact_path: "docs/tests/editor/editor-window-m3-workbench-900x620.png"
                .to_string(),
            meta_path: "zircon_editor/assets/ui/editor/workbench_page_chrome.zui".to_string(),
            adapter_key: "runtime-ui-template".to_string(),
            package_id: Some("zircon.editor.ui".to_string()),
            asset_unit: "single".to_string(),
            included_files: vec![
                "zircon_editor/assets/ui/editor/workbench_page_chrome.zui".to_string(),
                "zircon_editor/assets/ui/editor/asset_browser.zui".to_string(),
                "zircon_editor/assets/ui/editor/theme/editor_tokens.zui".to_string(),
            ],
            subassets: vec![
                asset_subasset(
                    "subasset-content-table",
                    "res://ui/editor/asset_browser.zui#AssetBrowserAssetTablePanel",
                    ResourceKind::UiWidget,
                ),
                asset_subasset(
                    "subasset-preview-card",
                    "res://ui/editor/asset_browser.zui#AssetBrowserContentPreviewCard",
                    ResourceKind::UiWidget,
                ),
            ],
            diagnostics: vec![
                "SVG icons resolve through scalable template metadata.".to_string(),
                "Retained-host content table uses workbench table row painter.".to_string(),
            ],
            resource_state: Some(ResourceState::Ready),
            resource_revision: Some(42),
            references: vec![
                asset_reference(
                    "ref-editor-base",
                    "res://ui/theme/editor_base.zui",
                    "editor_base.zui",
                    ResourceKind::UiStyle,
                ),
                asset_reference(
                    "ref-editor-material",
                    "res://ui/theme/editor_material.zui",
                    "editor_material.zui",
                    ResourceKind::UiStyle,
                ),
            ],
            used_by: vec![
                asset_reference(
                    "used-asset-browser",
                    "res://ui/editor/asset_browser.zui",
                    "Asset Browser",
                    ResourceKind::UiLayout,
                ),
                asset_reference(
                    "used-workbench-shell",
                    "res://ui/editor/host/workbench_shell.zui",
                    "Workbench Shell",
                    ResourceKind::UiLayout,
                ),
            ],
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
            recursive_asset_count: 6,
            depth: 0,
            selected: false,
        },
        AssetFolderSnapshot {
            folder_id: "folder-ui".to_string(),
            parent_folder_id: Some("folder-assets".to_string()),
            display_name: "ui".to_string(),
            recursive_asset_count: 4,
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
        AssetFolderSnapshot {
            folder_id: "folder-workbench".to_string(),
            parent_folder_id: Some("folder-ui".to_string()),
            display_name: "workbench".to_string(),
            recursive_asset_count: 4,
            depth: 2,
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

fn asset_reference(
    uuid: &str,
    locator: &str,
    display_name: &str,
    kind: ResourceKind,
) -> AssetReferenceSnapshot {
    AssetReferenceSnapshot {
        uuid: uuid.to_string(),
        locator: locator.to_string(),
        display_name: display_name.to_string(),
        kind: Some(kind),
        known_project_asset: true,
    }
}

fn asset_subasset(uuid: &str, locator: &str, kind: ResourceKind) -> AssetSubassetSnapshot {
    AssetSubassetSnapshot {
        uuid: uuid.to_string(),
        locator: locator.to_string(),
        kind,
        artifact_locator: Some(locator.to_string()),
        dependency_locators: Vec::new(),
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

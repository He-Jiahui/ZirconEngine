use super::*;

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

#[test]
#[ignore = "writes visual screenshot artifact for manual popup closeout"]
fn capture_scrolled_window_popup_visual_artifact() {
    std::env::set_var("SLINT_BACKEND", "software");

    let shell_size = ShellSizePx::new(900.0, 620.0);
    let metrics = WorkbenchChromeMetrics::default();
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
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
            hovered_menu_item_path: vec![17],
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
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
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

    let workbench = blend_space_workspace::support::blend_space_window(900, 620);
    blend_space_workspace::visual_capture::assert_blend_space_native_parent_chain_and_paint(
        &workbench, 900, 620,
    );
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
    assert_assets_drawer_adaptive_layout(&drawer, 900);
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
            scroll_offset: 0.0,
        },
    );
    let closed_presentation = ui.get_host_presentation();
    let closed_bytes = paint_host_frame_for_test(420, 260, &closed_presentation);

    set_host_page_overflow_visual_state(
        &ui,
        HostPageOverflowMenuStateData {
            open: true,
            hovered_page_index: 2,
            scroll_offset: 0.0,
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

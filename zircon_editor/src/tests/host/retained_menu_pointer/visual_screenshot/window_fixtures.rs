use super::fixture_support::{frame, visual_layout_output_path};
use super::*;

pub(super) fn workbench_fixture_window(width: u32, height: u32) -> UiHostWindow {
    workbench_fixture_window_with_presets(width, height, &[], None)
}

pub(super) fn workbench_fixture_window_with_presets(
    width: u32,
    height: u32,
    preset_names: &[String],
    active_preset_name: Option<&str>,
) -> UiHostWindow {
    let fixture = default_preview_fixture();
    presented_window_from_fixture(&fixture, width, height, preset_names, active_preset_name)
}

pub(in crate::tests::host::retained_menu_pointer) fn welcome_input_window(
    width: u32,
    height: u32,
) -> UiHostWindow {
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
        .with_workbench_slot(WorkbenchSlot::ExclusiveMainPage)
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
    data.console_output = "Welcome input commit preview: B".into();
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
            None,
        ),
        &fixture.layout,
        &fixture.descriptors,
        width,
        height,
        &[],
        None,
    )
}

pub(super) fn asset_browser_window(width: u32, height: u32) -> UiHostWindow {
    asset_browser_window_with_workspace(width, height, m3_asset_workspace())
}

pub(super) fn asset_browser_list_window(width: u32, height: u32) -> UiHostWindow {
    let mut workspace = m3_asset_workspace();
    workspace.view_mode = AssetViewMode::List;
    asset_browser_window_with_workspace(width, height, workspace)
}

pub(super) fn asset_browser_window_with_workspace(
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
    data.console_output = "Asset Browser M3 screenshot gate".into();
    data.project_path = "E:/Git/ZirconEngine".to_string();
    data.session_mode = EditorSessionMode::Project;
    data.project_open = true;
    presented_window_from_chrome(
        EditorChromeSnapshot::build(
            data,
            &fixture.layout,
            fixture.instances.clone(),
            fixture.descriptors.clone(),
            None,
        ),
        &fixture.layout,
        &fixture.descriptors,
        width,
        height,
        &[],
        None,
    )
}

pub(super) fn assert_asset_browser_compact_visual_layout(ui: &UiHostWindow) {
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

pub(super) fn assert_asset_browser_list_visual_layout(ui: &UiHostWindow) {
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

pub(super) fn presented_window_from_chrome(
    chrome: EditorChromeSnapshot,
    layout: &WorkbenchLayout,
    descriptors: &[ViewDescriptor],
    width: u32,
    height: u32,
    preset_names: &[String],
    active_preset_name: Option<&str>,
) -> UiHostWindow {
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
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

pub(super) fn workbench_window_bridge_for_visual_artifact(
    model: &WorkbenchViewModel,
    width: u32,
    height: u32,
) -> BuiltinWorkbenchWindowTemplateSurfaceBridge {
    let shell_size = UiSize::new(width as f32, height as f32);
    let runtime = Arc::new(
        load_startup_builtin_template_runtime()
            .expect("startup template runtime should load for screenshot"),
    );
    let host_bridge =
        BuiltinHostWindowTemplateBridge::new_with_runtime(runtime.clone(), shell_size)
            .expect("host template bridge should instantiate for screenshot");
    let mount_frame = host_bridge
        .root_shell_frames()
        .componentized_workbench_mount_frame(shell_size);
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new_mounted_with_runtime(runtime, mount_frame)
            .expect("workbench window template bridge should instantiate for screenshot");
    bridge
        .recompute_mounted_layout_with_workbench_model_at_scale(
            mount_frame,
            1.0,
            model,
            &WorkbenchChromeMetrics::default(),
        )
        .expect("workbench window template bridge should recompute screenshot layout");
    bridge
}

pub(super) fn set_host_page_overflow_visual_state(
    ui: &UiHostWindow,
    state: HostPageOverflowMenuStateData,
) {
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

pub(super) fn changed_snapshot_pixel_count_in_frame(
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

pub(super) fn assert_visible_workbench_layout_frames(
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

pub(super) fn release_first_document_tab_drag(ui: &UiHostWindow) {
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

pub(in crate::tests::host::retained_menu_pointer) fn save_window_snapshot(
    ui: &UiHostWindow,
    filename: &str,
) -> PathBuf {
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

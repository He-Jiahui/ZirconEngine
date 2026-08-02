pub(super) use std::collections::BTreeMap;

pub(super) use super::super::{
    apply_presentation as apply_presentation_with_module_plugins,
    apply_presentation_impl::to_host_contract_host_scene_data, pane_data_conversion,
};
pub(super) use crate::core::project::RecentProjectValidation;
pub(super) use crate::scene::modes::SceneModeActivation;
pub(super) use crate::scene::viewport::{
    DisplayMode, GridMode, ProjectionMode, TransformHandleKind, TransformSpace, ViewOrientation,
};
pub(super) use crate::ui::animation_editor::AnimationEditorPanePresentation;
pub(super) use crate::ui::asset_editor::UiAssetEditorPanePresentation;
pub(super) use crate::ui::layouts::common::model_rc;
pub(super) use crate::ui::layouts::views::blank_viewport_chrome;
pub(super) use crate::ui::layouts::windows::workbench_host_window::{
    self as host_window, collect_floating_windows as collect_floating_windows_with_module_plugins,
    document_pane as document_pane_with_module_plugins,
};
pub(super) use crate::ui::retained_host::callback_dispatch::BuiltinHostWindowTemplateBridge;
pub(super) use crate::ui::retained_host::floating_window_projection::{
    FloatingWindowProjectionBundle, build_floating_window_projection_bundle,
};
pub(super) use crate::ui::retained_host::shell_pointer::HostShellPointerRoute;
pub(super) use crate::ui::retained_host::tab_drag::host_shell_pointer_route_group_key;
pub(super) use crate::ui::template_runtime::{
    EditorUiCompatibilityHarness, EditorUiHostRuntime, UiComponentShowcaseDemoEventInput,
};
pub(super) use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
pub(super) use crate::ui::workbench::fixture::{PreviewFixture, default_preview_fixture};
pub(super) use crate::ui::workbench::layout::{DockEdge, MainHostPageLayout, WorkbenchLayout};
pub(super) use crate::ui::workbench::layout::{
    DocumentNode, FloatingWindowLayout, MainPageId, TabStackLayout,
};
pub(super) use crate::ui::workbench::model::WorkbenchViewModel;
pub(super) use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, EditorDataSnapshot, ProjectOverviewSnapshot,
};
pub(super) use crate::ui::workbench::startup::{
    EditorSessionMode, NewProjectFormSnapshot, RecentProjectItemSnapshot, WelcomePaneSnapshot,
};
pub(super) use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, ViewDescriptor,
    ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId, ViewKind, WorkbenchSlot,
};
pub(super) use zircon_runtime_interface::math::UVec2;
pub(super) use zircon_runtime_interface::ui::{
    component::{UiDragPayload, UiDragPayloadKind, UiDragSourceMetadata},
    layout::{UiFrame, UiSize},
};

pub(super) fn root_shell_fixture() -> (
    PreviewFixture,
    EditorChromeSnapshot,
    WorkbenchViewModel,
    BTreeMap<String, UiAssetEditorPanePresentation>,
    BTreeMap<String, AnimationEditorPanePresentation>,
) {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    (fixture, chrome, model, BTreeMap::new(), BTreeMap::new())
}

pub(super) fn welcome_shell_fixture() -> (
    EditorChromeSnapshot,
    WorkbenchViewModel,
    BTreeMap<String, UiAssetEditorPanePresentation>,
    BTreeMap<String, AnimationEditorPanePresentation>,
) {
    let descriptors = vec![
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.welcome"),
            ViewKind::ActivityWindow,
            "Welcome",
        )
        .with_workbench_slot(WorkbenchSlot::ExclusiveMainPage)
        .with_icon_key("welcome"),
    ];
    let welcome_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.welcome#1"),
        descriptor_id: ViewDescriptorId::new("editor.welcome"),
        title: "Welcome".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::ExclusivePage(MainPageId::new("page:welcome")),
    };
    let chrome = EditorChromeSnapshot::build(
        EditorDataSnapshot {
            scene_entries: Default::default(),
            inspector: None,
            status_line: "Ready".to_string(),
            console_output: "Ready".into(),
            status_task_progress: None,
            hovered_axis: None,
            viewport_size: UVec2::new(1280, 720),
            scene_viewport_settings: crate::scene::viewport::SceneViewportChromeSettings::default(),
            mesh_import_path: String::new(),
            project_overview: ProjectOverviewSnapshot::default(),
            asset_activity: AssetWorkspaceSnapshot::default(),
            asset_browser: AssetWorkspaceSnapshot::default(),
            project_path: String::new(),
            session_mode: EditorSessionMode::Welcome,
            welcome: WelcomePaneSnapshot {
                title: "Open or Create".to_string(),
                subtitle: "Recent projects and a renderable empty-project template".to_string(),
                status_message: "No recent project".to_string(),
                browse_supported: false,
                recent_projects: vec![RecentProjectItemSnapshot {
                    display_name: "Broken".to_string(),
                    path: "E:/Missing/Broken".to_string(),
                    validation: RecentProjectValidation::Missing,
                    last_opened_label: "Just now".to_string(),
                    selected: true,
                }],
                form: NewProjectFormSnapshot {
                    project_name: "WelcomeProject".to_string(),
                    location: "E:/Work".to_string(),
                    project_path_preview: "E:/Work/WelcomeProject".to_string(),
                    template_label: "Renderable Empty".to_string(),
                    can_create: true,
                    can_open_existing: true,
                    validation_message: String::new(),
                },
            },
            project_open: false,
            can_undo: false,
            can_redo: false,
            bridge_diagnostics: Default::default(),
        },
        &WorkbenchLayout {
            active_main_page: MainPageId::new("page:welcome"),
            main_pages: vec![MainHostPageLayout::ExclusiveActivityWindowPage {
                id: MainPageId::new("page:welcome"),
                title: "Welcome".to_string(),
                window_instance: welcome_instance.instance_id.clone(),
            }],
            drawers: BTreeMap::new(),
            activity_windows: Default::default(),
            floating_windows: Vec::new(),
            region_overrides: BTreeMap::new(),
            view_overrides: BTreeMap::new(),
        },
        vec![welcome_instance],
        descriptors,
        None,
    );
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    (chrome, model, BTreeMap::new(), BTreeMap::new())
}

pub(super) fn apply_presentation(
    ui: &crate::ui::retained_host::UiHostWindow,
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    geometry: &WorkbenchShellGeometry,
    preset_names: &[String],
    active_preset_name: Option<&str>,
    ui_asset_panes: &BTreeMap<String, UiAssetEditorPanePresentation>,
    animation_panes: &BTreeMap<String, AnimationEditorPanePresentation>,
    runtime_diagnostics: Option<&zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot>,
    _shared_root_frames: Option<
        &crate::ui::retained_host::callback_dispatch::BuiltinHostRootShellFrames,
    >,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
) {
    apply_presentation_with_workbench_layout_frames(
        ui,
        model,
        chrome,
        geometry,
        preset_names,
        active_preset_name,
        ui_asset_panes,
        animation_panes,
        runtime_diagnostics,
        _shared_root_frames,
        crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames::default(),
        floating_window_projection_bundle,
    );
}

pub(super) fn apply_presentation_with_workbench_layout_frames(
    ui: &crate::ui::retained_host::UiHostWindow,
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    geometry: &WorkbenchShellGeometry,
    preset_names: &[String],
    active_preset_name: Option<&str>,
    ui_asset_panes: &BTreeMap<String, UiAssetEditorPanePresentation>,
    animation_panes: &BTreeMap<String, AnimationEditorPanePresentation>,
    runtime_diagnostics: Option<&zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot>,
    _shared_root_frames: Option<
        &crate::ui::retained_host::callback_dispatch::BuiltinHostRootShellFrames,
    >,
    componentized_workbench_layout_frames: crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
) {
    apply_presentation_with_module_plugins(
        ui,
        model,
        chrome,
        geometry,
        preset_names,
        active_preset_name,
        ui_asset_panes,
        animation_panes,
        runtime_diagnostics,
        &host_window::ModulePluginsPaneViewData::default(),
        &host_window::BuildExportPaneViewData::default(),
        None,
        None,
        componentized_workbench_layout_frames,
        floating_window_projection_bundle,
        None,
    );
}

pub(super) fn frame_rect_from_ui_frame(frame: UiFrame) -> crate::ui::retained_host::FrameRect {
    crate::ui::retained_host::FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

pub(super) fn document_pane(
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    ui_asset_panes: &BTreeMap<String, UiAssetEditorPanePresentation>,
    animation_panes: &BTreeMap<String, AnimationEditorPanePresentation>,
    runtime_diagnostics: Option<&zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot>,
) -> host_window::PaneData {
    document_pane_with_module_plugins(
        model,
        chrome,
        ui_asset_panes,
        animation_panes,
        runtime_diagnostics,
        &host_window::ModulePluginsPaneViewData::default(),
        &host_window::BuildExportPaneViewData::default(),
    )
}

pub(super) fn collect_floating_windows(
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    geometry: &WorkbenchShellGeometry,
    ui_asset_panes: &BTreeMap<String, UiAssetEditorPanePresentation>,
    animation_panes: &BTreeMap<String, AnimationEditorPanePresentation>,
    runtime_diagnostics: Option<&zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot>,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
) -> Vec<host_window::FloatingWindowData> {
    collect_floating_windows_with_module_plugins(
        model,
        chrome,
        geometry,
        ui_asset_panes,
        animation_panes,
        runtime_diagnostics,
        &host_window::ModulePluginsPaneViewData::default(),
        &host_window::BuildExportPaneViewData::default(),
        floating_window_projection_bundle,
    )
}

pub(super) fn host_frame_rect(x: f32, y: f32, width: f32, height: f32) -> host_window::FrameRect {
    host_window::FrameRect {
        x,
        y,
        width,
        height,
    }
}

pub(super) fn template_frame(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> crate::ui::layouts::views::ViewTemplateFrameData {
    crate::ui::layouts::views::ViewTemplateFrameData {
        x,
        y,
        width,
        height,
    }
}

pub(super) fn mount_node(
    node_id: &str,
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> crate::ui::layouts::views::ViewTemplateNodeData {
    crate::ui::layouts::views::ViewTemplateNodeData {
        node_id: node_id.into(),
        control_id: control_id.into(),
        role: "Mount".into(),
        text: "".into(),
        dispatch_kind: "".into(),
        action_id: "".into(),
        surface_variant: "".into(),
        text_tone: "".into(),
        button_variant: "".into(),
        font_size: 0.0,
        font_weight: 0,
        text_align: "left".into(),
        overflow: "".into(),
        corner_radius: 0.0,
        border_width: 0.0,
        frame: template_frame(x, y, width, height),
        ..Default::default()
    }
}

pub(super) fn host_tabs(
    ids: &[&str],
) -> crate::ui::retained_host::primitives::ModelRc<host_window::TabData> {
    model_rc(
        ids.iter()
            .enumerate()
            .map(|(index, id)| host_window::TabData {
                id: (*id).into(),
                slot: format!("slot-{index}").into(),
                title: format!("Tab {index}").into(),
                icon_key: "tab".into(),
                active: index == 0,
                closeable: true,
            })
            .collect(),
    )
}

pub(super) fn host_chrome_tab_frames(
    ids: &[&str],
) -> crate::ui::retained_host::primitives::ModelRc<host_window::HostChromeTabData> {
    let tabs = host_tabs(ids);
    model_rc(
        (0..tabs.row_count())
            .filter_map(|row| {
                let tab = tabs.row_data(row)?;
                let x = 8.0 + row as f32 * 94.0;
                Some(host_window::HostChromeTabData {
                    control_id: format!("DockTab{row}").into(),
                    tab,
                    frame: host_frame_rect(x, 1.0, 92.0, 30.0),
                    close_frame: host_frame_rect(x + 68.0, 8.0, 16.0, 16.0),
                })
            })
            .collect(),
    )
}

pub(super) fn host_chrome_menu_frames(
    count: usize,
) -> crate::ui::retained_host::primitives::ModelRc<host_window::HostChromeControlFrameData> {
    model_rc(
        (0..count)
            .map(|row| host_window::HostChromeControlFrameData {
                control_id: format!("MenuSlot{row}").into(),
                frame: host_frame_rect(8.0 + row as f32 * 42.0, 2.0, 40.0, 22.0),
            })
            .collect(),
    )
}

pub(super) fn host_pane(id: &str, title: &str) -> host_window::PaneData {
    host_window::PaneData {
        id: id.into(),
        slot: format!("{id}-slot").into(),
        kind: format!("{title}Kind").into(),
        title: title.into(),
        icon_key: format!("{title}-icon").into(),
        subtitle: format!("{title} subtitle").into(),
        info: format!("{title} info").into(),
        show_empty: false,
        empty_title: format!("{title} empty").into(),
        empty_body: format!("{title} body").into(),
        primary_action_label: format!("{title} primary").into(),
        primary_action_id: format!("{title}.primary").into(),
        secondary_action_label: format!("{title} secondary").into(),
        secondary_action_id: format!("{title}.secondary").into(),
        secondary_hint: format!("{title} hint").into(),
        show_toolbar: true,
        viewport: blank_viewport_chrome(),
        native_body: host_window::PaneNativeBodyData {
            hierarchy: host_window::HierarchyPaneViewData::default(),
            inspector: host_window::InspectorPaneViewData::default(),
            console: host_window::ConsolePaneViewData::default(),
            assets_activity: host_window::AssetsActivityPaneViewData::default(),
            asset_browser: host_window::AssetBrowserPaneViewData::default(),
            project_overview: host_window::ProjectOverviewPaneViewData::default(),
            performance_timeline: host_window::PerformanceTimelinePaneViewData::default(),
            module_plugins: host_window::ModulePluginsPaneViewData::default(),
            build_export: host_window::BuildExportPaneViewData::default(),
            generated_bottom: host_window::GeneratedBottomPaneViewData::default(),
            ui_asset: UiAssetEditorPanePresentation::default(),
            animation: host_window::AnimationEditorPaneViewData::default(),
        },
        pane_presentation: None,
    }
}
